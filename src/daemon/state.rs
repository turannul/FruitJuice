use std::collections::HashMap;
use std::process::Child;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::battery::{BatteryStats, query_battery, spawn_battery_observer};
use crate::companion::{CompanionDeviceData, query_all_companions, query_companion_battery};
use crate::models::{convert_model_readable, query_model_info};
use crate::sysfs::{SysfsNodes, remove_sysfs_device, update_sysfs_device};

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
}

pub struct DeviceState {
    pub dev_name: String,
    pub display_name: String,
    pub model_readable: String,
    pub class: String,
    pub connections: HashMap<u32, String>,
    pub last_poll: Instant,
    pub last_cap: u8,
    pub last_status: String,
    pub observer: Option<Child>,
    pub parent_udid: Option<String>,
    pub companions: Vec<String>,
}

impl DeviceState {
    pub fn log_battery_changes(&mut self, stats: &BatteryStats) {
        let mut changes = Vec::new();
        if stats.cap != self.last_cap {
            changes.push(format!("{}% >> {}%", self.last_cap, stats.cap));
        }
        if stats.status != self.last_status {
            changes.push(format!("State: {} >> {}", self.last_status, stats.status));
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

    fn register_or_update_companion(&mut self, parent_udid: &str, comp_data: &CompanionDeviceData) {
        let watch_udid = comp_data.udid.clone();
        let readable_model = convert_model_readable(&comp_data.model_code);
        let class = "Watch".to_string();
        let dev_name = format!("fj_{}_{}", class, watch_udid);
        let status = if comp_data.is_charging {
            "Charging".to_string()
        } else if comp_data.battery_cap == 100 {
            "Full".to_string()
        } else {
            "Discharging".to_string()
        };

        let stats = BatteryStats {
            cap: comp_data.battery_cap,
            status: status.clone(),
            voltage: 4000,
            cycles: 0,
            charge_full_design: 1000,
            charge_full: 1000,
            charge_now: (comp_data.battery_cap as i32) * 10,
            serial: comp_data.serial.clone(),
        };

        if let Some(state) = self.devices.get_mut(&watch_udid) {
            update_sysfs_device(
                &state.dev_name,
                &state.class,
                &watch_udid,
                &state.display_name,
                &stats,
                &self.sysfs_nodes,
                &mut self.registered_devices,
            );
            state.log_battery_changes(&stats);
        } else {
            println!(
                "[NEW] {} ({}) {}% [{}]",
                comp_data.device_name, readable_model, comp_data.battery_cap, status
            );
            update_sysfs_device(
                &dev_name,
                &class,
                &watch_udid,
                &comp_data.device_name,
                &stats,
                &self.sysfs_nodes,
                &mut self.registered_devices,
            );
            self.devices.insert(
                watch_udid,
                DeviceState {
                    dev_name,
                    display_name: comp_data.device_name.clone(),
                    model_readable: readable_model,
                    class,
                    connections: HashMap::new(),
                    last_poll: Instant::now(),
                    last_cap: comp_data.battery_cap,
                    last_status: status,
                    observer: None,
                    parent_udid: Some(parent_udid.to_string()),
                    companions: Vec::new(),
                },
            );
        }
    }

    pub fn handle_attached(
        &mut self,
        dev_id: u32,
        udid: String,
        conn_type: String,
        tx: &mpsc::Sender<DaemonEvent>,
    ) {
        self.dev_to_udid.insert(dev_id, udid.clone());

        if let Some(state) = self.devices.get_mut(&udid) {
            let prev_conn = if state.connections.values().any(|c| c == "USB") {
                "Wired"
            } else {
                "Wireless"
            };
            state.connections.insert(dev_id, conn_type);
            let new_conn = if state.connections.values().any(|c| c == "USB") {
                "Wired"
            } else {
                "Wireless"
            };
            let is_network = new_conn == "Wireless";

            if prev_conn != new_conn {
                println!(
                    "[CHG] {} Connection: {} >> {}",
                    state.display_name, prev_conn, new_conn
                );
                if let Some(mut child) = state.observer.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                state.observer = spawn_battery_observer(&udid, is_network, tx.clone());

                if new_conn == "Wired"
                    && let Some(stats) = query_battery(&udid, is_network)
                {
                    update_sysfs_device(
                        &state.dev_name,
                        &state.class,
                        &udid,
                        &state.display_name,
                        &stats,
                        &self.sysfs_nodes,
                        &mut self.registered_devices,
                    );
                    state.log_battery_changes(&stats);
                    state.last_poll = Instant::now();
                }
            }
        } else {
            let is_network = conn_type == "Network";
            let (display_name, readable_model, class) = query_model_info(&udid, is_network);
            let dev_name = format!("fj_{}_{}", class, udid);
            if let Some(stats) = query_battery(&udid, is_network) {
                let health = if stats.charge_full_design > 0 {
                    (stats.charge_full as f32 / stats.charge_full_design as f32) * 100.0
                } else {
                    0.0
                };
                println!(
                    "[NEW] {} ({}) {}% [{}] (Health: {:.1}%, {} cycles)",
                    display_name, readable_model, stats.cap, stats.status, health, stats.cycles
                );
                update_sysfs_device(
                    &dev_name,
                    &class,
                    &udid,
                    &display_name,
                    &stats,
                    &self.sysfs_nodes,
                    &mut self.registered_devices,
                );

                let observer = spawn_battery_observer(&udid, is_network, tx.clone());

                let mut connections = HashMap::new();
                connections.insert(dev_id, conn_type);

                let companions = query_all_companions(&udid);
                let mut companion_udids = Vec::new();
                for comp in &companions {
                    companion_udids.push(comp.udid.clone());
                }

                self.devices.insert(
                    udid.clone(),
                    DeviceState {
                        dev_name,
                        display_name,
                        model_readable: readable_model,
                        class,
                        connections,
                        last_poll: Instant::now(),
                        last_cap: stats.cap,
                        last_status: stats.status.clone(),
                        observer,
                        parent_udid: None,
                        companions: companion_udids,
                    },
                );

                for comp in companions {
                    self.register_or_update_companion(&udid, &comp);
                }
            }
        }
    }

    pub fn handle_detached(&mut self, dev_id: u32, tx: &mpsc::Sender<DaemonEvent>) {
        if let Some(udid) = self.dev_to_udid.remove(&dev_id)
            && let Some(state) = self.devices.get_mut(&udid)
        {
            let prev_conn = if state.connections.values().any(|c| c == "USB") {
                "Wired"
            } else {
                "Wireless"
            };
            state.connections.remove(&dev_id);
            if state.connections.is_empty() {
                println!("[DEL] {} ({})", state.display_name, state.model_readable);
                if let Some(mut child) = state.observer.take() {
                    let _ = child.kill();
                    let _ = child.wait();
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
            } else {
                let new_conn = if state.connections.values().any(|c| c == "USB") {
                    "Wired"
                } else {
                    "Wireless"
                };
                if prev_conn != new_conn {
                    println!(
                        "[CHG] {} {} >> {}",
                        state.display_name, prev_conn, new_conn
                    );
                    if let Some(mut child) = state.observer.take() {
                        let _ = child.kill();
                        let _ = child.wait();
                    }
                    let is_network = new_conn == "Wireless";
                    state.observer = spawn_battery_observer(&udid, is_network, tx.clone());
                }
            }
        }
    }

    pub fn handle_battery_push(&mut self, udid: &str) {
        let mut comp_udids = Vec::new();
        if let Some(state) = self.devices.get_mut(udid)
            && state.last_poll.elapsed() >= Duration::from_millis(300)
        {
            state.last_poll = Instant::now();
            let is_network = !state.connections.values().any(|c| c == "USB");
            if let Some(stats) = query_battery(udid, is_network) {
                update_sysfs_device(
                    &state.dev_name,
                    &state.class,
                    udid,
                    &state.display_name,
                    &stats,
                    &self.sysfs_nodes,
                    &mut self.registered_devices,
                );
                state.log_battery_changes(&stats);
            }
            comp_udids = state.companions.clone();
        }

        for c_udid in comp_udids {
            if let Some((cap, is_charging)) = query_companion_battery(udid, &c_udid) {
                let status = if is_charging {
                    "Charging".to_string()
                } else if cap == 100 {
                    "Full".to_string()
                } else {
                    "Discharging".to_string()
                };
                let stats = BatteryStats {
                    cap,
                    status,
                    voltage: 4000,
                    cycles: 0,
                    charge_full_design: 1000,
                    charge_full: 1000,
                    charge_now: (cap as i32) * 10,
                    serial: String::new(),
                };
                if let Some(comp_state) = self.devices.get_mut(&c_udid) {
                    update_sysfs_device(
                        &comp_state.dev_name,
                        &comp_state.class,
                        &c_udid,
                        &comp_state.display_name,
                        &stats,
                        &self.sysfs_nodes,
                        &mut self.registered_devices,
                    );
                    comp_state.log_battery_changes(&stats);
                }
            }
        }
    }

    pub fn handle_periodic_poll(&mut self, interval: u64) {
        let mut parent_to_comps: Vec<(String, Vec<String>)> = Vec::new();
        for (udid, state) in self.devices.iter_mut() {
            if state.parent_udid.is_none()
                && state.last_poll.elapsed() >= Duration::from_secs(interval)
            {
                state.last_poll = Instant::now();
                let is_network = !state.connections.values().any(|c| c == "USB");
                if let Some(stats) = query_battery(udid, is_network) {
                    update_sysfs_device(
                        &state.dev_name,
                        &state.class,
                        udid,
                        &state.display_name,
                        &stats,
                        &self.sysfs_nodes,
                        &mut self.registered_devices,
                    );
                    state.log_battery_changes(&stats);
                }
                if !state.companions.is_empty() {
                    parent_to_comps.push((udid.clone(), state.companions.clone()));
                }
            }
        }

        for (p_udid, comps) in parent_to_comps {
            for c_udid in comps {
                if let Some((cap, is_charging)) = query_companion_battery(&p_udid, &c_udid) {
                    let status = if is_charging {
                        "Charging".to_string()
                    } else if cap == 100 {
                        "Full".to_string()
                    } else {
                        "Discharging".to_string()
                    };
                    let stats = BatteryStats {
                        cap,
                        status,
                        voltage: 4000,
                        cycles: 0,
                        charge_full_design: 1000,
                        charge_full: 1000,
                        charge_now: (cap as i32) * 10,
                        serial: String::new(),
                    };
                    if let Some(comp_state) = self.devices.get_mut(&c_udid) {
                        update_sysfs_device(
                            &comp_state.dev_name,
                            &comp_state.class,
                            &c_udid,
                            &comp_state.display_name,
                            &stats,
                            &self.sysfs_nodes,
                            &mut self.registered_devices,
                        );
                        comp_state.log_battery_changes(&stats);
                    }
                }
            }
        }
    }
}
