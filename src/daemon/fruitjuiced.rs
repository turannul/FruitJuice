use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;

use fruitjuice::netmux::run_usbmuxd_listener;
use fruitjuice::state::{DaemonEvent, DeviceManager};
use fruitjuice::sysfs::SysfsNodes;

fn print_help() {
    println!("Usage: fruitjuiced [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help    Print help information");
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if env::args().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        exit(0);
    }

    let a_node = "/sys/kernel/fruitjuice/add_device";
    let r_node = "/sys/kernel/fruitjuice/remove_device";
    let u_node = "/sys/kernel/fruitjuice/update_device";
    let sysfs_nodes = SysfsNodes {
        add: a_node,
        update: u_node,
    };

    if !Path::new(a_node).exists() {
        eprintln!("[ERR] module not found. Is fruitjuice module loaded?");
        exit(1);
    }

    let registered_devices: Vec<String> = fs::read_dir("/sys/class/power_supply/")
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|name| name.starts_with("fj_"))
                .collect()
        })
        .unwrap_or_default();

    let mut manager = DeviceManager::new(sysfs_nodes, r_node, registered_devices);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let tx_netmux = tx.clone();
    tokio::spawn(async move {
        run_usbmuxd_listener(tx_netmux).await;
    });

    while let Some(ev) = rx.recv().await {
        match ev {
            DaemonEvent::Attached {
                dev_id,
                udid,
                conn_type,
            } => {
                manager.handle_attached(dev_id, udid, conn_type, &tx).await;
            }
            DaemonEvent::Detached { dev_id } => {
                manager.handle_detached(dev_id, &tx);
            }
            DaemonEvent::BatteryPush { udid } => {
                manager.handle_battery_push(&udid).await;
            }
            DaemonEvent::MuxDisconnected => {
                manager.handle_mux_disconnected();
            }
        }
    }
}
