use idevice::provider::UsbmuxdProvider;
use idevice::usbmuxd::UsbmuxdAddr;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use idevice::IdeviceService;
use idevice::services::lockdown::LockdownClient;

use crate::battery::{BatteryStats, query_battery, spawn_battery_observer};
use crate::companion::{
    CompanionDeviceData, mask_udid, query_all_companions, query_companion_battery,
};
use crate::sysfs::{SysfsDeviceUpdate, SysfsNodes, remove_sysfs_device, update_sysfs_device};

fn conn_mode(connections: &HashMap<u32, String>) -> &'static str {
    match connections.values().any(|c| c == "USB") {
        true => "Wired",
        false => "Wireless",
    }
}

pub struct DeviceInfo {
    pub display_name: String,
    pub model_name: String,
    pub model_detail: String,
    pub hardware_model: String,
    pub class: String,
    pub os_version: String,
    pub product_type: String,
}

pub async fn query_device_info(
    provider: &UsbmuxdProvider,
    self_reported_model: Option<&str>,
) -> DeviceInfo {
    let mut dev_name = None;
    let mut product_type = None;
    let mut hardware_model = None;
    let mut os_version = None;
    let mut dev_class = None;

    match LockdownClient::connect(provider).await {
        Ok(mut lockdown) => {
            if let Some(dict) = lockdown
                .get_value(None, None)
                .await
                .ok()
                .and_then(|v| v.into_dictionary())
            {
                let extract = |k: &str| dict.get(k).and_then(|v| v.as_string()).map(String::from);
                dev_name = extract("DeviceName");
                product_type = extract("ProductType");
                hardware_model = extract("HardwareModel");
                os_version = extract("ProductVersion");
                dev_class = extract("DeviceClass");
            } else {
                for (key, target) in [
                    ("DeviceName", &mut dev_name),
                    ("ProductType", &mut product_type),
                    ("HardwareModel", &mut hardware_model),
                    ("ProductVersion", &mut os_version),
                    ("DeviceClass", &mut dev_class),
                ] {
                    *target = lockdown
                        .get_value(Some(key), None)
                        .await
                        .ok()
                        .and_then(|v| v.as_string().map(String::from));
                }
            }
        }
        Err(e) => {
            eprintln!(
                "[ERR] Failed to connect LockdownClient for [...{}]: {}",
                mask_udid(&provider.udid),
                e
            );
        }
    }

    let dev_name = dev_name.unwrap_or_else(|| "iPhone".to_string());
    let product_type = product_type.unwrap_or_default();
    let hardware_model = hardware_model.unwrap_or_default();
    let os_version = os_version.unwrap_or_default();
    let dev_class = dev_class.unwrap_or_else(|| "iPhone".to_string());

    let model_name = self_reported_model
        .map(String::from)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if !product_type.is_empty() {
                product_type.clone()
            } else {
                dev_class.clone()
            }
        });

    let class = match () {
        _ if model_name.contains("iPad") || dev_class == "iPad" => "iPad",
        _ if model_name.contains("Watch") || dev_class == "Watch" => "Watch",
        _ => "iPhone",
    }
    .to_string();

    let display_name = match dev_name.as_str() {
        "iPhone" | "iPad" | "Apple Watch" => model_name.clone(),
        _ => dev_name,
    };

    DeviceInfo {
        display_name,
        model_name,
        model_detail: "N/A".to_string(),
        hardware_model,
        class,
        os_version,
        product_type,
    }
}

pub enum DaemonEvent {
    Attached {
        dev_id: u32,
        udid: String,
        conn_type: String,
    },
    Detached {
        dev_id: u32,
    },
    BatteryPush {
        udid: String,
    },
    MuxDisconnected,
}

pub struct DeviceState {
    pub dev_name: String,
    pub display_name: String,
    pub model_readable: String,
    pub model_detail: String,
    pub hardware_model: String,
    pub class: String,
    pub os_version: String,
    pub product_type: String,
    pub provider: UsbmuxdProvider,
    pub connections: HashMap<u32, String>,
    pub last_cap: u8,
    pub last_status: String,
    pub observer: Option<JoinHandle<()>>,
    pub companions: Vec<String>,
}

impl DeviceState {
    pub fn to_sysfs_update<'a>(
        &'a self,
        udid: &'a str,
        stats: &'a BatteryStats,
    ) -> SysfsDeviceUpdate<'a> {
        SysfsDeviceUpdate {
            dev_name: &self.dev_name,
            class: &self.class,
            udid,
            device_name: &self.display_name,
            device_model: &self.model_readable,
            model_detail: &self.model_detail,
            hardware_model: &self.hardware_model,
            os_version: &self.os_version,
            product_type: &self.product_type,
            stats,
        }
    }

    pub fn log_battery_changes(&mut self, stats: &BatteryStats) {
        let mut changes = Vec::new();
        if stats.cap != self.last_cap {
            changes.push(format!("{}% >> {}%", self.last_cap, stats.cap));
        }
        if stats.status != self.last_status {
            changes.push(format!("{} >> {}", self.last_status, stats.status));
        }
        if !changes.is_empty() {
            println!("[CHG] {} {}", self.display_name, changes.join(" "));
            self.last_cap = stats.cap;
            self.last_status = stats.status.clone();
        }
    }
}

pub struct DeviceManager<'a> {
    devices: HashMap<String, DeviceState>,
    dev_to_udid: HashMap<u32, String>,
    registered_devices: Vec<String>,
    sysfs_nodes: SysfsNodes<'a>,
    remove_node: &'a str,
}

impl<'a> DeviceManager<'a> {
    pub fn new(
        sysfs_nodes: SysfsNodes<'a>,
        remove_node: &'a str,
        initial_registered: Vec<String>,
    ) -> Self {
        Self {
            devices: HashMap::new(),
            dev_to_udid: HashMap::new(),
            registered_devices: initial_registered,
            sysfs_nodes,
            remove_node,
        }
    }

    fn register_or_update_companion(
        &mut self,
        comp_data: &CompanionDeviceData,
        parent_provider: &UsbmuxdProvider,
    ) {
        let watch_udid = comp_data.udid.clone();
        let model_name = comp_data.model_name.clone();
        let model_detail = comp_data.model_detail.clone();
        let hw_model = comp_data.hardware_model.clone();
        let class = "Watch".to_string();
        let dev_name = format!("fj_{}_{}", class, watch_udid);

        let cap = match comp_data.battery_cap {
            0 => self
                .devices
                .get(&watch_udid)
                .map(|d| d.last_cap)
                .unwrap_or(0),
            c => c,
        };

        let stats = BatteryStats::companion_synthetic(
            cap,
            comp_data.is_charging,
            comp_data.serial.clone(),
            Some(model_name.clone()),
        );

        if let Some(state) = self.devices.get_mut(&watch_udid) {
            let update = state.to_sysfs_update(&watch_udid, &stats);
            update_sysfs_device(&update, &self.sysfs_nodes, &mut self.registered_devices);
            state.log_battery_changes(&stats);
            return;
        }

        println!(
            "[NEW] {} ({}) {}% [{}]",
            comp_data.device_name, model_name, cap, stats.status
        );
        let update = SysfsDeviceUpdate {
            dev_name: &dev_name,
            class: &class,
            udid: &watch_udid,
            device_name: &comp_data.device_name,
            device_model: &model_name,
            model_detail: &model_detail,
            hardware_model: &hw_model,
            os_version: &comp_data.product_version,
            product_type: &comp_data.model_code,
            stats: &stats,
        };
        update_sysfs_device(&update, &self.sysfs_nodes, &mut self.registered_devices);
        self.devices.insert(
            watch_udid,
            DeviceState {
                dev_name,
                display_name: comp_data.device_name.clone(),
                model_readable: model_name,
                model_detail,
                hardware_model: hw_model,
                class,
                os_version: comp_data.product_version.clone(),
                product_type: comp_data.model_code.clone(),
                provider: parent_provider.clone(),
                connections: HashMap::new(),
                last_cap: cap,
                last_status: stats.status,
                observer: None,
                companions: Vec::new(),
            },
        );
    }

    pub async fn handle_attached(
        &mut self,
        dev_id: u32,
        udid: String,
        conn_type: String,
        tx: &mpsc::UnboundedSender<DaemonEvent>,
    ) {
        self.dev_to_udid.insert(dev_id, udid.clone());

        let provider = UsbmuxdProvider {
            addr: UsbmuxdAddr::default(),
            tag: 0,
            udid: udid.clone(),
            device_id: dev_id,
            label: "fruitjuiced".to_string(),
        };

        if let Some(state) = self.devices.get_mut(&udid) {
            let prev_conn = conn_mode(&state.connections);
            state.connections.insert(dev_id, conn_type);
            state.provider = provider.clone();
            let new_conn = conn_mode(&state.connections);

            if prev_conn != new_conn {
                println!(
                    "[CHG] {} Connection: {} >> {}",
                    state.display_name, prev_conn, new_conn
                );
                if let Some(h) = state.observer.take() {
                    h.abort();
                }
                state.observer = Some(spawn_battery_observer(provider.clone(), tx.clone()));

                if new_conn == "Wired"
                    && let Some(stats) = query_battery(&provider).await
                {
                    let update = state.to_sysfs_update(&udid, &stats);
                    update_sysfs_device(&update, &self.sysfs_nodes, &mut self.registered_devices);
                    state.log_battery_changes(&stats);
                }
            }
            return;
        }

        let battery_stats = query_battery(&provider).await;
        let self_model = battery_stats
            .as_ref()
            .and_then(|s| s.self_reported_model.as_deref());

        let model_info = query_device_info(&provider, self_model).await;
        let dev_name = format!("fj_{}_{}", model_info.class, udid);

        let Some(stats) = battery_stats else {
            return;
        };

        let health_str = stats.format_health();

        println!(
            "[NEW] {} ({}) {}% [{}] (Health: {}, {} cycles, OS: {})",
            model_info.display_name,
            model_info.model_name,
            stats.cap,
            stats.status,
            health_str,
            stats.cycles,
            model_info.os_version
        );

        let update = SysfsDeviceUpdate {
            dev_name: &dev_name,
            class: &model_info.class,
            udid: &udid,
            device_name: &model_info.display_name,
            device_model: &model_info.model_name,
            model_detail: &model_info.model_detail,
            hardware_model: &model_info.hardware_model,
            os_version: &model_info.os_version,
            product_type: &model_info.product_type,
            stats: &stats,
        };
        update_sysfs_device(&update, &self.sysfs_nodes, &mut self.registered_devices);

        let observer = Some(spawn_battery_observer(provider.clone(), tx.clone()));
        let mut connections = HashMap::new();
        connections.insert(dev_id, conn_type);

        let companions = query_all_companions(&provider).await;
        let companion_udids: Vec<String> = companions.iter().map(|c| c.udid.clone()).collect();

        self.devices.insert(
            udid.clone(),
            DeviceState {
                dev_name,
                display_name: model_info.display_name,
                model_readable: model_info.model_name,
                model_detail: model_info.model_detail,
                hardware_model: model_info.hardware_model,
                class: model_info.class,
                os_version: model_info.os_version,
                product_type: model_info.product_type,
                provider: provider.clone(),
                connections,
                last_cap: stats.cap,
                last_status: stats.status.clone(),
                observer,
                companions: companion_udids,
            },
        );

        for comp in companions {
            self.register_or_update_companion(&comp, &provider);
        }
    }

    pub fn handle_detached(&mut self, dev_id: u32, tx: &mpsc::UnboundedSender<DaemonEvent>) {
        let Some(udid) = self.dev_to_udid.remove(&dev_id) else {
            return;
        };
        let Some(state) = self.devices.get_mut(&udid) else {
            return;
        };

        let prev_conn = conn_mode(&state.connections);
        state.connections.remove(&dev_id);

        if state.connections.is_empty() {
            println!("[DEL] {} ({})", state.display_name, state.model_readable);
            if let Some(h) = state.observer.take() {
                h.abort();
            }
            remove_sysfs_device(
                &state.dev_name,
                self.remove_node,
                &mut self.registered_devices,
            );
            let comp_list = state.companions.clone();
            self.devices.remove(&udid);
            for c_udid in comp_list {
                if let Some(comp_state) = self.devices.remove(&c_udid) {
                    println!(
                        "[DEL] {} ({})",
                        comp_state.display_name, comp_state.model_readable
                    );
                    remove_sysfs_device(
                        &comp_state.dev_name,
                        self.remove_node,
                        &mut self.registered_devices,
                    );
                }
            }
            return;
        }

        if let Some(&remaining_dev_id) = state.connections.keys().next() {
            state.provider.device_id = remaining_dev_id;
        }
        let new_conn = conn_mode(&state.connections);
        if prev_conn != new_conn {
            println!("[CHG] {} {} >> {}", state.display_name, prev_conn, new_conn);
            if let Some(h) = state.observer.take() {
                h.abort();
            }
            state.observer = Some(spawn_battery_observer(state.provider.clone(), tx.clone()));
        }
    }

    async fn refresh_device_battery(
        &mut self,
        udid: &str,
    ) -> Option<(UsbmuxdProvider, Vec<String>)> {
        let state = self.devices.get_mut(udid)?;
        let provider = state.provider.clone();
        if let Some(stats) = query_battery(&provider).await {
            let update = state.to_sysfs_update(udid, &stats);
            update_sysfs_device(&update, &self.sysfs_nodes, &mut self.registered_devices);
            state.log_battery_changes(&stats);
        }
        Some((provider, state.companions.clone()))
    }

    async fn refresh_companion_battery(&mut self, provider: &UsbmuxdProvider, c_udid: &str) {
        let Some((mut cap, is_charging)) = query_companion_battery(provider, c_udid).await else {
            return;
        };

        if cap == 0 {
            cap = self.devices.get(c_udid).map(|d| d.last_cap).unwrap_or(0);
        }

        let stats = BatteryStats::companion_synthetic(cap, is_charging, String::new(), None);
        let Some(comp_state) = self.devices.get_mut(c_udid) else {
            return;
        };

        let update = comp_state.to_sysfs_update(c_udid, &stats);
        update_sysfs_device(&update, &self.sysfs_nodes, &mut self.registered_devices);
        comp_state.log_battery_changes(&stats);
    }

    pub async fn handle_battery_push(&mut self, udid: &str) {
        let Some((provider, companions)) = self.refresh_device_battery(udid).await else {
            return;
        };

        for c_udid in companions {
            self.refresh_companion_battery(&provider, &c_udid).await;
        }
    }

    pub fn handle_mux_disconnected(&mut self) {
        for (_, mut state) in self.devices.drain() {
            if let Some(h) = state.observer.take() {
                h.abort();
            }
            remove_sysfs_device(
                &state.dev_name,
                self.remove_node,
                &mut self.registered_devices,
            );
        }
        self.dev_to_udid.clear();
    }
}
