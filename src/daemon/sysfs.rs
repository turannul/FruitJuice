use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crate::battery::BatteryStats;

pub struct SysfsNodes<'a> {
    pub add: &'a str,
    pub update: &'a str,
}

pub fn safe_write(path: &str, val: &str) {
    if let Err(e) = fs::write(path, val) {
        eprintln!("[ERR] Failed to write to {}: {}", path, e);
    }
}

pub fn update_sysfs_device(
    dev_name: &str,
    class: &str,
    udid: &str,
    model_name: &str,
    stats: &BatteryStats,
    nodes: &SysfsNodes,
    registered: &mut Vec<String>,
) {
    let base_path = format!("/sys/class/power_supply/{}", dev_name);

    if !Path::new(&base_path).exists() {
        if let Err(e) = fs::write(nodes.add, format!("{} {}", class, udid)) {
            eprintln!("[ERR] Failed to register {}: {}", dev_name, e);
            return;
        }
        registered.push(dev_name.to_string());
        sleep(Duration::from_millis(50));
    }

    let status_str = stats.status.replace(' ', "_");
    let model_str = model_name.replace(' ', "_");
    let payload = format!(
        "name={} cap={} status={} present=1 voltage={} cycles={} full_design={} full={} now={} serial={} model={}\n",
        dev_name,
        stats.cap,
        status_str,
        stats.voltage,
        stats.cycles,
        stats.charge_full_design,
        stats.charge_full,
        stats.charge_now,
        stats.serial,
        model_str
    );

    safe_write(nodes.update, &payload);
}

pub fn remove_sysfs_device(dev_name: &str, r_node: &str, registered: &mut Vec<String>) {
    if let Some(pos) = registered.iter().position(|x| x == dev_name) {
        registered.remove(pos);
    }
    safe_write(r_node, dev_name);
}
