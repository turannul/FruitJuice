mod battery;
mod companion;
mod models;
mod netmux;
mod state;
mod sysfs;
mod xml;

use std::env;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::exit;
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;

use netmux::connect_and_listen;
use state::{DaemonEvent, DeviceManager};
use sysfs::SysfsNodes;
use xml::{extract_xml_int, extract_xml_string};

fn print_help() {
    println!("FruitJuice: iDevice Battery Bridge Daemon");
    println!();
    println!("Usage: fruitjuiced [OPTIONS]");
    println!();
    println!("Options:");
    println!(
        "  -r, --refresh <SECONDS>  Fallback battery poll interval in seconds (default: 300, 0 to disable)"
    );
    println!("  -h, --help               Print help information");
}

fn main() {
    let mut interval = 300;
    let args: Vec<String> = env::args().collect();
    let mut arg_idx = 1;
    while arg_idx < args.len() {
        match args[arg_idx].as_str() {
            "-r" | "--refresh" => {
                if let Some(val) = args.get(arg_idx + 1)
                    && let Ok(v) = val.parse::<u64>()
                {
                    interval = v;
                    arg_idx += 1;
                }
            }
            "-h" | "--help" => {
                print_help();
                exit(0);
            }
            _ => {}
        }
        arg_idx += 1;
    }

    let a_node = "/sys/kernel/fruitjuice/add_device";
    let r_node = "/sys/kernel/fruitjuice/remove_device";
    let u_node = "/sys/kernel/fruitjuice/update_device";
    let sysfs_nodes = SysfsNodes {
        add: a_node,
        update: u_node,
    };

    if !Path::new(a_node).exists() {
        eprintln!("[ERR] Driver not found. Is fruitjuice module loaded?");
        exit(1);
    }

    let mut registered_devices: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply/") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str()
                && name.starts_with("fj_")
            {
                registered_devices.push(name.to_string());
            }
        }
    }

    let mut manager = DeviceManager::new(sysfs_nodes, r_node, registered_devices);
    let (tx, rx) = mpsc::channel();

    let tx_netmux = tx.clone();
    thread::spawn(move || {
        loop {
            let mut stream = match connect_and_listen() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "[ERR] Failed to connect to netmuxd: {}. Retrying in 3s...",
                        e
                    );
                    sleep(Duration::from_secs(3));
                    continue;
                }
            };

            loop {
                let mut hdr = [0u8; 16];
                if stream.read_exact(&mut hdr).is_err() {
                    eprintln!("[ERR] netmuxd connection lost. Reconnecting...");
                    break;
                }
                let msg_len = u32::from_le_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as usize;
                if msg_len < 16 {
                    continue;
                }
                let mut body = vec![0u8; msg_len - 16];
                if stream.read_exact(&mut body).is_err() {
                    eprintln!("[ERR] netmuxd connection lost. Reconnecting...");
                    break;
                }
                let xml = String::from_utf8_lossy(&body);
                let msg_type = extract_xml_string(&xml, "MessageType");

                if msg_type == "Attached" {
                    let dev_id = extract_xml_int(&xml, "DeviceID") as u32;
                    let udid = extract_xml_string(&xml, "SerialNumber");
                    let conn_type = extract_xml_string(&xml, "ConnectionType");
                    let _ = tx_netmux.send(DaemonEvent::Attached {
                        dev_id,
                        udid,
                        conn_type,
                    });
                } else if msg_type == "Detached" {
                    let dev_id = extract_xml_int(&xml, "DeviceID") as u32;
                    let _ = tx_netmux.send(DaemonEvent::Detached { dev_id });
                }
            }
            sleep(Duration::from_secs(2));
        }
    });

    loop {
        let event = if interval > 0 {
            match rx.recv_timeout(Duration::from_secs(interval)) {
                Ok(ev) => Some(ev),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        } else {
            match rx.recv() {
                Ok(ev) => Some(ev),
                Err(_) => break,
            }
        };

        if let Some(ev) = event {
            match ev {
                DaemonEvent::Attached {
                    dev_id,
                    udid,
                    conn_type,
                } => {
                    manager.handle_attached(dev_id, udid, conn_type, &tx);
                }
                DaemonEvent::Detached { dev_id } => {
                    manager.handle_detached(dev_id, &tx);
                }
                DaemonEvent::BatteryPush { udid } => {
                    manager.handle_battery_push(&udid);
                }
            }
        } else if interval > 0 {
            manager.handle_periodic_poll(interval);
        }
    }
}
