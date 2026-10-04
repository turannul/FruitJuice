use idevice::IdeviceService;
use idevice::provider::UsbmuxdProvider;
use idevice::services::diagnostics_relay::DiagnosticsRelayClient;
use idevice::services::notification_proxy::NotificationProxyClient;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::state::DaemonEvent;

#[derive(Debug, Clone)]
pub struct BatteryStats {
    pub cap: u8,
    pub status: String,
    pub serial: String,
    pub voltage: i32,
    pub cycles: i32,
    pub charge_full_design: i32,
    pub charge_full: i32,
    pub charge_now: i32,
    pub current_now: i32,
    pub power_now: i32,
    pub time_to_empty: i32,
    pub time_to_full: i32,
    pub health_percent: i32,
    pub raw_capacity: i32,
    pub adapter_watts: i32,
    pub adapter_voltage: i32,
    pub is_wireless: bool,
    pub self_reported_model: Option<String>,
}

impl BatteryStats {
    pub fn format_health(&self) -> String {
        match (self.charge_full_design, self.charge_full) {
            (d, f) if d > 0 && f > 0 => format!("{:.1}%", (f as f64 / d as f64) * 100.0),
            _ if self.health_percent >= 0 => format!("{}%", self.health_percent),
            _ => "N/A".to_string(),
        }
    }

    pub fn companion_synthetic(
        cap: u8,
        is_charging: bool,
        serial: String,
        model: Option<String>,
    ) -> Self {
        let status = match (is_charging, cap) {
            (true, _) => "Charging",
            (false, 100) => "Full",
            _ => "Discharging",
        }
        .to_string();
        Self {
            cap,
            status,
            voltage: 4_000_000,
            cycles: 0,
            charge_full_design: 1_000_000,
            charge_full: 1_000_000,
            charge_now: (cap as i32) * 10_000,
            current_now: 0,
            power_now: 0,
            time_to_empty: 0,
            time_to_full: 0,
            health_percent: 100,
            raw_capacity: cap as i32,
            adapter_watts: 0,
            adapter_voltage: 0,
            is_wireless: false,
            serial,
            self_reported_model: model,
        }
    }
}

pub async fn query_battery(provider: &UsbmuxdProvider) -> Option<BatteryStats> {
    let mut diag = match DiagnosticsRelayClient::connect(provider).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[ERR] DiagnosticsRelayClient connect failed for [...{}]: {}",
                crate::companion::mask_udid(&provider.udid),
                e
            );
            return None;
        }
    };

    // 1. Query IORegistry "product" for self-reported model name
    let self_reported_model =
        if let Ok(Some(prod)) = diag.ioregistry(None, Some("product"), None).await {
            prod.get("product-name")
                .or_else(|| prod.get("product-description"))
                .and_then(|v| match v {
                    plist::Value::Data(d) => {
                        let s = String::from_utf8_lossy(d)
                            .trim_matches('\0')
                            .trim()
                            .to_string();
                        if s.is_empty() { None } else { Some(s) }
                    }
                    plist::Value::String(s) => {
                        let s = s.trim().to_string();
                        if s.is_empty() { None } else { Some(s) }
                    }
                    _ => None,
                })
        } else {
            None
        };

    // 2. Query IORegistry "AppleSmartBattery" for full battery telemetry
    let diag_dict = match diag.ioregistry(None, Some("AppleSmartBattery"), None).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            eprintln!(
                "[ERR] AppleSmartBattery entry returned None for [...{}]",
                crate::companion::mask_udid(&provider.udid)
            );
            return None;
        }
        Err(e) => {
            eprintln!(
                "[ERR] Failed to query AppleSmartBattery for [...{}]: {}",
                crate::companion::mask_udid(&provider.udid),
                e
            );
            return None;
        }
    };

    let cap = diag_dict
        .get("CurrentCapacity")
        .and_then(|v| v.as_unsigned_integer())
        .unwrap_or(0) as u8;

    let serial = diag_dict
        .get("Serial")
        .and_then(|v| v.as_string())
        .unwrap_or_default()
        .to_string();

    let is_charging = diag_dict
        .get("IsCharging")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    let is_connected = diag_dict
        .get("ExternalConnected")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    let is_full = diag_dict
        .get("FullyCharged")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    let status = match () {
        _ if is_full || cap >= 100 => "Full",
        _ if is_charging => "Charging",
        _ if is_connected => "Not charging",
        _ => "Discharging",
    };

    let voltage = diag_dict
        .get("Voltage")
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32
        * 1000;

    let cycles = diag_dict
        .get("CycleCount")
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32;

    let amperage = diag_dict
        .get("Amperage")
        .or_else(|| diag_dict.get("InstantAmperage"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32;
    let current_now = amperage * 1000;

    let battery_power = diag_dict
        .get("BatteryPower")
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32;
    let power_now = battery_power.abs() * 1000;

    let time_remaining = diag_dict
        .get("TimeRemaining")
        .or_else(|| diag_dict.get("AvgTimeToEmpty"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32;

    let rem_secs = (time_remaining * 60).max(0);
    let (time_to_empty, time_to_full) = match is_charging {
        true => (0, rem_secs),
        false => (rem_secs, 0),
    };

    let bdata = diag_dict.get("BatteryData").and_then(|v| v.as_dictionary());

    let charge_full_design = bdata
        .and_then(|d| d.get("DesignCapacity"))
        .or_else(|| diag_dict.get("DesignCapacity"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32
        * 1000;

    let charge_full = bdata
        .and_then(|d| d.get("NominalChargeCapacity"))
        .or_else(|| diag_dict.get("NominalChargeCapacity"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32
        * 1000;

    let charge_now = bdata
        .and_then(|d| d.get("RemainingCapacity"))
        .or_else(|| diag_dict.get("AppleRawCurrentCapacity"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32
        * 1000;

    let raw_capacity = bdata
        .and_then(|d| d.get("AbsoluteCapacity"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32;

    let health_percent = match (charge_full_design, charge_full) {
        (d, f) if d > 0 && f > 0 => ((f as f64 / d as f64) * 100.0).round() as i32,
        _ => -1,
    };

    let adapter = diag_dict
        .get("AdapterDetails")
        .and_then(|v| v.as_dictionary());

    let adapter_watts = adapter
        .and_then(|d| d.get("Watts"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32;

    let adapter_voltage = adapter
        .and_then(|d| d.get("AdapterVoltage"))
        .and_then(|v| v.as_signed_integer())
        .unwrap_or(0) as i32;

    let is_wireless = adapter
        .and_then(|d| d.get("IsWireless"))
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    Some(BatteryStats {
        cap,
        status: status.to_string(),
        serial,
        voltage,
        cycles,
        charge_full_design,
        charge_full,
        charge_now,
        current_now,
        power_now,
        time_to_empty,
        time_to_full,
        health_percent,
        raw_capacity,
        adapter_watts,
        adapter_voltage,
        is_wireless,
        self_reported_model,
    })
}

pub fn spawn_battery_observer(
    provider: UsbmuxdProvider,
    tx: mpsc::UnboundedSender<DaemonEvent>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut notif = match NotificationProxyClient::connect(&provider).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[ERR] Failed to connect notification proxy for [...{}]: {}",
                    crate::companion::mask_udid(&provider.udid),
                    e
                );
                return;
            }
        };

        if let Err(e) = notif
            .observe_notification("com.apple.system.powersources")
            .await
        {
            eprintln!(
                "[WARN] Failed to observe powersources for [...{}]: {}",
                crate::companion::mask_udid(&provider.udid),
                e
            );
        }
        if let Err(e) = notif
            .observe_notification("com.apple.system.powersources.percent")
            .await
        {
            eprintln!(
                "[WARN] Failed to observe percent for [...{}]: {}",
                crate::companion::mask_udid(&provider.udid),
                e
            );
        }

        loop {
            match notif.receive_notification().await {
                Ok(name) => {
                    if name.starts_with("com.apple.system.powersources") {
                        let _ = tx.send(DaemonEvent::BatteryPush {
                            udid: provider.udid.clone(),
                        });
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[WARN] NotificationProxy connection closed for [...{}]: {}",
                        crate::companion::mask_udid(&provider.udid),
                        e
                    );
                    break;
                }
            }
        }
    })
}
