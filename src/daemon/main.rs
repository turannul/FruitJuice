use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::thread::sleep;
use std::time::Duration;

pub struct BatteryInfo {
    pub cap: u8,
    pub status: String,
    pub name: String,
    pub serial: String,
    pub udid: String,
    pub class: String,
    pub voltage: i32,
    pub cycles: i32,
    pub charge_full_design: i32,
    pub charge_full: i32,
    pub charge_now: i32,
}

fn extract_xml_int(xml: &str, key: &str) -> i32 {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let rest = &xml[pos + key_tag.len()..];
        if let Some(int_start) = rest.find("<integer>") {
            let val_start = int_start + "<integer>".len();
            if let Some(int_end) = rest[val_start..].find("</integer>") {
                return rest[val_start..val_start + int_end]
                    .trim()
                    .parse()
                    .unwrap_or(0);
            }
        }
    }
    0
}

fn extract_xml_string(xml: &str, key: &str) -> String {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let rest = &xml[pos + key_tag.len()..];
        if let Some(s_start) = rest.find("<string>") {
            let val_start = s_start + "<string>".len();
            if let Some(s_end) = rest[val_start..].find("</string>") {
                return rest[val_start..val_start + s_end].trim().to_string();
            }
        }
    }
    String::new()
}

fn extract_xml_bool(xml: &str, key: &str) -> bool {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let rest = &xml[pos + key_tag.len()..];
        let trimmed = rest.trim_start();
        if trimmed.starts_with("<true/>") {
            return true;
        }
    }
    false
}

fn convert_model_readable(code: &str) -> String {
    let code_upper = code.to_uppercase();
    match code_upper.as_str() {
        "D23AP" => "iPhone Air".to_string(),
        "V53AP" => "iPhone 17 Pro".to_string(),
        "V54AP" => "iPhone 17 Pro Max".to_string(),
        "V57AP" => "iPhone 17".to_string(),
        "V159AP" => "iPhone 17e".to_string(),
        "D93AP" => "iPhone 16 Pro".to_string(),
        "D94AP" => "iPhone 16 Pro Max".to_string(),
        "D42AP" => "iPhone 16".to_string(),
        "D43AP" => "iPhone 16 Plus".to_string(),
        "D83AP" => "iPhone 15 Pro".to_string(),
        "D84AP" => "iPhone 15 Pro Max".to_string(),
        "D37AP" => "iPhone 15".to_string(),
        "D38AP" => "iPhone 15 Plus".to_string(),
        "D73AP" => "iPhone 14 Pro".to_string(),
        "D74AP" => "iPhone 14 Pro Max".to_string(),
        "D27AP" => "iPhone 14".to_string(),
        "D28AP" => "iPhone 14 Plus".to_string(),
        "D63AP" => "iPhone 13 Pro".to_string(),
        "D64AP" => "iPhone 13 Pro Max".to_string(),
        "D16AP" => "iPhone 13 mini".to_string(),
        "D17AP" => "iPhone 13".to_string(),
        "D53PAP" | "D53P" => "iPhone 12 Pro".to_string(),
        "D54PAP" | "D54P" => "iPhone 12 Pro Max".to_string(),
        "D52GAP" => "iPhone 12 mini".to_string(),
        "D53GAP" => "iPhone 12".to_string(),
        "N104AP" => "iPhone 11".to_string(),
        "D421AP" => "iPhone 11 Pro".to_string(),
        "D431AP" => "iPhone 11 Pro Max".to_string(),
        "D22AP" | "D221AP" => "iPhone X".to_string(),
        "D321AP" => "iPhone XS".to_string(),
        "D331PAP" | "D331AP" => "iPhone XS Max".to_string(),
        "N841AP" => "iPhone XR".to_string(),
        "N69AP" | "N69UAP" => "iPhone SE (1st Gen)".to_string(),
        "D79AP" => "iPhone SE (2nd Gen)".to_string(),
        "D49AP" => "iPhone SE (3rd Gen)".to_string(),
        "J817" | "J818" | "J817AP" | "J818AP" => "iPad Pro 11-inch (M4)".to_string(),
        "J820" | "J821" | "J820AP" | "J821AP" => "iPad Pro 13-inch (M4)".to_string(),
        "J617" | "J618" | "J617AP" | "J618AP" => "iPad Pro 11-inch (4th Gen)".to_string(),
        "J620" | "J621" | "J620AP" | "J621AP" => "iPad Pro 12.9-inch (6th Gen)".to_string(),
        "J517" | "J518" | "J517AP" | "J518AP" => "iPad Pro 11-inch (3rd Gen)".to_string(),
        "J522" | "J523" | "J522AP" | "J523AP" => "iPad Pro 12.9-inch (5th Gen)".to_string(),
        _ => {
            if code.is_empty() {
                "iPhone".to_string()
            } else {
                code.to_string()
            }
        }
    }
}

pub fn get_device_info(debug: bool) -> Vec<BatteryInfo> {
    let mut devices = Vec::new();
    let id_output = Command::new("idevice_id").arg("-n").output().ok();
    let ids = if let Some(out) = id_output {
        if out.status.success() {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            return devices;
        }
    } else {
        return devices;
    };

    for line in ids.lines() {
        let udid = line.split_whitespace().next().unwrap_or("");
        if udid.is_empty() {
            continue;
        }

        let diag_output = Command::new("idevicediagnostics")
            .args(&["-u", udid, "ioregentry", "AppleSmartBattery", "--network"])
            .output()
            .ok();

        let diag_xml = if let Some(out) = diag_output {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else {
                continue;
            }
        } else {
            continue;
        };

        let model_output = Command::new("idevicediagnostics")
            .args(&["-u", udid, "ioreg", "IOService", "--network"])
            .output()
            .ok();

        let mut internal_model = String::new();
        if let Some(out) = model_output {
            if out.status.success() {
                let model_xml = String::from_utf8_lossy(&out.stdout);
                if let Some(pos) = model_xml.find("IOPlatformExpertDevice") {
                    internal_model = extract_xml_string(&model_xml[pos..], "name");
                }
            }
        }

        if internal_model.is_empty() {
            continue;
        }

        let name_output = Command::new("idevice_id").arg(udid).output().ok();
        let device_name = if let Some(out) = name_output {
            if out.status.success() {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            } else {
                "iPhone".to_string()
            }
        } else {
            "iPhone".to_string()
        };

        let cap = extract_xml_int(&diag_xml, "StateOfCharge") as u8;
        let serial = extract_xml_string(&diag_xml, "Serial");
        let is_charging = extract_xml_bool(&diag_xml, "IsCharging");
        let is_connected = extract_xml_bool(&diag_xml, "ExternalConnected");
        let is_full = extract_xml_bool(&diag_xml, "FullyCharged");

        let status = if is_full || cap >= 100 {
            "Full"
        } else if is_charging {
            "Charging"
        } else if is_connected {
            "Not charging"
        } else {
            "Discharging"
        };

        let voltage = extract_xml_int(&diag_xml, "Voltage") * 1000;
        let cycles = extract_xml_int(&diag_xml, "CycleCount");
        let charge_full_design = extract_xml_int(&diag_xml, "DesignCapacity") * 1000;
        let charge_full = extract_xml_int(&diag_xml, "NominalChargeCapacity") * 1000;
        let charge_now = extract_xml_int(&diag_xml, "AppleRawCurrentCapacity") * 1000;

        let readable_model = convert_model_readable(&internal_model);
        let class = if readable_model.contains("iPad") {
            "iPad".to_string()
        } else {
            "iPhone".to_string()
        };
        let is_generic = device_name == "iPhone" || device_name == "iPad" || device_name.is_empty();

        let health = if charge_full_design > 0 {
            (charge_full as f32 / charge_full_design as f32) * 100.0
        } else {
            0.0
        };

        if debug {
            if is_generic {
                println!("Found: {}", readable_model);
            } else {
                println!("Found: {} ({})", device_name, readable_model);
            }
            println!(
                "Battery Data: {}% (Health: {:.2}%), {} cycles",
                cap, health, cycles
            );
        }

        devices.push(BatteryInfo {
            cap,
            status: status.to_string(),
            name: if is_generic {
                readable_model
            } else {
                device_name
            },
            serial,
            udid: udid.to_string(),
            class,
            voltage,
            cycles,
            charge_full_design,
            charge_full,
            charge_now,
        });
    }
    devices
}

fn main() {
    let a_node = "/sys/kernel/fruitjuice/add_device";
    let r_node = "/sys/kernel/fruitjuice/remove_device";

    if !Path::new(a_node).exists() {
        eprintln!("Driver not found. Is fruitjuice module loaded?");
        exit(1);
    }

    let mut interval = 180;
    let mut debug = false;
    let args: Vec<String> = env::args().collect();
    let mut arg_idx = 1;
    while arg_idx < args.len() {
        match args[arg_idx].as_str() {
            "--refresh" => {
                if let Some(val) = args.get(arg_idx + 1) {
                    if let Ok(v) = val.parse::<u64>() {
                        interval = v;
                        arg_idx += 1;
                    }
                }
            }
            "--debug" => debug = true,
            _ => {}
        }
        arg_idx += 1;
    }

    if debug {
        println!("FruitJuice Daemon started. Polling every {}s...", interval);
    }

    let mut registered_devices: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply/") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("network_") {
                    registered_devices.push(name.to_string());
                }
            }
        }
    }

    loop {
        let current_data = get_device_info(debug);

        let mut idx = 0;
        while idx < registered_devices.len() {
            let still_alive = current_data
                .iter()
                .any(|d| format!("network_{}_{}", d.class, d.udid) == registered_devices[idx]);

            if !still_alive {
                let dev_to_remove = registered_devices.remove(idx);
                println!("Device lost or unresponsive, removing: {}", dev_to_remove);
                let _ = fs::write(r_node, &dev_to_remove);
            } else {
                idx += 1;
            }
        }

        for info in current_data {
            let dev_name = format!("network_{}_{}", info.class, info.udid);
            let base_path = format!("/sys/class/power_supply/{}", dev_name);

            if !Path::new(&base_path).exists() {
                if let Err(e) = fs::write(a_node, format!("{} {}", info.class, info.udid)) {
                    eprintln!("Failed to register {}: {}", dev_name, e);
                    continue;
                }
                registered_devices.push(dev_name.clone());
            }

            let _ = fs::write(format!("{}/model_name_sync", base_path), info.name);
            let _ = fs::write(format!("{}/serial_number_sync", base_path), info.serial);
            let _ = fs::write(format!("{}/capacity", base_path), info.cap.to_string());
            let _ = fs::write(format!("{}/status", base_path), info.status);
            let _ = fs::write(
                format!("{}/voltage_now", base_path),
                info.voltage.to_string(),
            );
            let _ = fs::write(
                format!("{}/cycle_count", base_path),
                info.cycles.to_string(),
            );
            let _ = fs::write(
                format!("{}/charge_full_design", base_path),
                info.charge_full_design.to_string(),
            );
            let _ = fs::write(
                format!("{}/charge_full", base_path),
                info.charge_full.to_string(),
            );
            let _ = fs::write(
                format!("{}/charge_now", base_path),
                info.charge_now.to_string(),
            );
            let _ = fs::write(format!("{}/present", base_path), "1");
        }
        sleep(Duration::from_secs(interval));
    }
}
