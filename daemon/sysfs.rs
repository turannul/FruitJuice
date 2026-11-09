use std::fs;
use std::path::Path;

use crate::battery::BatteryStats;

pub struct SysfsNodes<'a> {
    pub add: &'a str,
    pub update: &'a str,
}

pub struct SysfsDeviceUpdate<'a> {
    pub dev_name: &'a str,
    pub class: &'a str,
    pub udid: &'a str,
    pub device_name: &'a str,
    pub device_model: &'a str,
    pub model_detail: &'a str,
    pub hardware_model: &'a str,
    pub os_version: &'a str,
    pub product_type: &'a str,
    pub stats: &'a BatteryStats,
}

pub fn safe_write(path: &str, val: &str) {
    if let Err(e) = fs::write(path, val) {
        eprintln!("[ERR] Failed to write to {}: {}", path, e);
    }
}

pub fn update_sysfs_device(
    device: &SysfsDeviceUpdate<'_>,
    nodes: &SysfsNodes<'_>,
    registered: &mut Vec<String>,
) {
    let base_path = format!("/sys/class/power_supply/{}", device.dev_name);

    if !Path::new(&base_path).exists() {
        if let Err(e) = fs::write(nodes.add, format!("{} {}", device.class, device.udid)) {
            eprintln!("[ERR] Failed to register {}: {}", device.dev_name, e);
            return;
        }
        registered.push(device.dev_name.to_string());
    }

    let payload = format!(
        "name={} udid={} cap={} status=\"{}\" present=1 voltage={} cycles={} full_design={} full={} now={} \
         current={} power={} time_empty={} time_full={} health_pct={} raw_cap={} watts={} \
         adapter_v={} wireless={} serial=\"{}\" model=\"{}\" device_model=\"{}\" model_detail=\"{}\" hw_model=\"{}\" os_ver=\"{}\" prod_type=\"{}\"\n",
        device.dev_name,
        device.udid,
        device.stats.cap,
        device.stats.status,
        device.stats.voltage,
        device.stats.cycles,
        device.stats.charge_full_design,
        device.stats.charge_full,
        device.stats.charge_now,
        device.stats.current_now,
        device.stats.power_now,
        device.stats.time_to_empty,
        device.stats.time_to_full,
        device.stats.health_percent,
        device.stats.raw_capacity,
        device.stats.adapter_watts,
        device.stats.adapter_voltage,
        if device.stats.is_wireless { 1 } else { 0 },
        device.stats.serial,
        device.device_name,
        device.device_model,
        device.model_detail,
        device.hardware_model,
        device.os_version,
        device.product_type,
    );

    safe_write(nodes.update, &payload);
}

pub fn remove_sysfs_device(dev_name: &str, r_node: &str, registered: &mut Vec<String>) {
    if let Some(pos) = registered.iter().position(|x| x == dev_name) {
        registered.remove(pos);
    }
    safe_write(r_node, dev_name);
}
