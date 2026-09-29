use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::state::DaemonEvent;
use crate::xml::{extract_xml_bool, extract_xml_int, extract_xml_string};

pub struct BatteryStats {
    pub cap: u8,
    pub status: String,
    pub serial: String,
    pub voltage: i32,
    pub cycles: i32,
    pub charge_full_design: i32,
    pub charge_full: i32,
    pub charge_now: i32,
}

pub fn query_battery(udid: &str, is_network: bool) -> Option<BatteryStats> {
    let mut diag_cmd = Command::new("idevicediagnostics");
    if is_network {
        diag_cmd.arg("--network");
    }
    diag_cmd.args(["-u", udid, "ioregentry", "AppleSmartBattery"]);
    let diag_output = diag_cmd.output().ok()?;
    if !diag_output.status.success() {
        let err_msg = String::from_utf8_lossy(&diag_output.stderr);
        let detail = err_msg.trim();
        if !detail.is_empty() {
            eprintln!("[ERR] idevicediagnostics failed for {}: {}", udid, detail);
        } else {
            eprintln!("[ERR] idevicediagnostics failed for {}", udid);
        }
        return None;
    }
    let diag_xml = String::from_utf8_lossy(&diag_output.stdout).to_string();

    let cap = extract_xml_int(&diag_xml, "CurrentCapacity") as u8;
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

    Some(BatteryStats {
        cap,
        status: status.to_string(),
        serial,
        voltage,
        cycles,
        charge_full_design,
        charge_full,
        charge_now,
    })
}

pub fn spawn_battery_observer(
    udid: &str,
    is_network: bool,
    tx: mpsc::Sender<DaemonEvent>,
) -> Option<Child> {
    let mut cmd = Command::new("stdbuf");
    cmd.args(["-oL", "idevicenotificationproxy", "-u", udid]);
    if is_network {
        cmd.arg("-n");
    }
    cmd.args([
        "observe",
        "com.apple.system.powersources",
        "com.apple.system.powersources.percent",
    ]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => {
            let mut direct = Command::new("idevicenotificationproxy");
            direct.args(["-u", udid]);
            if is_network {
                direct.arg("-n");
            }
            direct.args([
                "observe",
                "com.apple.system.powersources",
                "com.apple.system.powersources.percent",
            ]);
            direct.stdout(Stdio::piped());
            direct.stderr(Stdio::null());
            match direct.spawn() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "[ERR] Failed to spawn notification observer for {}: {}",
                        udid, e
                    );
                    return None;
                }
            }
        }
    };

    let stdout = child.stdout.take()?;
    let udid_clone = udid.to_string();

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line {
                let trimmed = l.trim();
                if trimmed.starts_with('>') {
                    let _ = tx.send(DaemonEvent::BatteryPush {
                        udid: udid_clone.clone(),
                    });
                }
            } else {
                break;
            }
        }
    });

    Some(child)
}
