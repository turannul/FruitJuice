use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::thread::sleep;
use std::time::Duration;

pub fn get_iphone_data() -> Option<(u8, String, String, String, String, String)> {
    let gen_output = Command::new("ideviceinfo").arg("--network").output().ok()?;
    if !gen_output.status.success() {
        return None;
    }

    let batt_output = Command::new("ideviceinfo")
        .args(&["--network", "--domain", "com.apple.mobile.battery"])
        .output()
        .ok()?;
    if !batt_output.status.success() {
        return None;
    }

    let bulk_info = format!(
        "{}\n{}",
        String::from_utf8_lossy(&gen_output.stdout),
        String::from_utf8_lossy(&batt_output.stdout)
    );

    let mut cap = None;
    let mut is_charging = false;
    let mut is_connected = false;
    let mut is_full = false;
    let mut name = String::new();
    let mut serial = String::new();
    let mut udid = String::new();
    let mut prod_type = String::new();
    let mut class = String::new();

    for line in bulk_info.lines() {
        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        match key {
            "BatteryCurrentCapacity" => cap = value.parse::<u8>().ok(),
            "BatteryIsCharging" => is_charging = value.to_lowercase() == "true",
            "ExternalConnected" => is_connected = value.to_lowercase() == "true",
            "FullyCharged" => is_full = value.to_lowercase() == "true",
            "DeviceName" => name = value.to_string(),
            "SerialNumber" => serial = value.to_string(),
            "UniqueDeviceID" => udid = value.to_string(),
            "ProductType" => prod_type = value.to_string(),
            "DeviceClass" => class = value.to_string(),
            _ => {}
        }
    }

    if let Some(c) = cap {
        let status = if is_full || c >= 100 {
            "Full"
        } else if is_charging {
            "Charging"
        } else if is_connected {
            "Not charging"
        } else {
            "Discharging"
        };
        Some((c, status.to_string(), name, serial, udid, class))
    } else {
        None
    }
}

fn main() {
    let f_node = "/sys/kernel/fruitjuice/add_device";

    if !Path::new(f_node).exists() {
        eprintln!("ERR: FruitJuice driver not found at {}. Is the module loaded?", f_node);
        exit(1);
    }

    loop {
        if let Some((cap, status, name, serial, udid, class)) = get_iphone_data() {
            let dev_name = format!("network_{}_{}", class, udid);
            let base_path = format!("/sys/class/power_supply/{}", dev_name);

            if !Path::new(&base_path).exists() {
                println!("Registering new device: {}", dev_name);
                if let Err(e) = fs::write(f_node, format!("{} {}", class, udid)) {
                    eprintln!("ERR: Failed to register device: {}", e);
                    exit(1);
                }
            }

            let target_cap = format!("{}/capacity", base_path);
            let target_status = format!("{}/status", base_path);
            let target_model = format!("{}/model_name_sync", base_path);
            let target_serial = format!("{}/serial_number_sync", base_path);
            let target_present = format!("{}/present", base_path);

            let _ = fs::write(&target_model, name);
            let _ = fs::write(&target_serial, serial);
            let _ = fs::write(&target_cap, cap.to_string());
            let _ = fs::write(&target_status, status);
            let _ = fs::write(&target_present, "1");
        }
        sleep(Duration::from_secs(180));
    }
}
