use idevice::usbmuxd::{RawPacket, UsbmuxdAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

use crate::state::DaemonEvent;

fn parse_network_address(addr: &[u8]) -> Option<std::net::IpAddr> {
    let family = match addr {
        [0x10 | 0x1c, f, ..] => *f,
        [f, ..] => *f,
        [] => return None,
    };

    match (family, addr) {
        (2, [_, _, _, _, b0, b1, b2, b3, ..]) => Some(std::net::IpAddr::V4(
            std::net::Ipv4Addr::new(*b0, *b1, *b2, *b3),
        )),
        _ => None,
    }
}

pub async fn run_usbmuxd_listener(tx: mpsc::UnboundedSender<DaemonEvent>) {
    loop {
        let addr = UsbmuxdAddr::from_env_var().unwrap_or_default();
        let mut socket = match addr.to_socket().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "[ERR] Failed to connect to usbmuxd/netmuxd: {}. Retrying in 5s...",
                    e
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        // Send Listen request (version: 1 (XML), message: 8 (PLIST), tag: 1)
        let mut req = plist::Dictionary::new();
        req.insert("MessageType".into(), "Listen".into());
        req.insert("ClientVersionString".into(), "fruitjuice".into());
        req.insert("kLibUSBMuxVersion".into(), 3.into());

        let raw: Vec<u8> = RawPacket::new(req, 1, 8, 1).into();
        if let Err(e) = socket.write_all(&raw).await {
            eprintln!(
                "[ERR] Failed to send Listen command to usbmuxd: {}. Retrying in 5s...",
                e
            );
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        // Read handshake response
        let mut header = [0u8; 16];
        if let Err(e) = socket.read_exact(&mut header).await {
            eprintln!(
                "[ERR] Failed to read Listen handshake header: {}. Retrying in 5s...",
                e
            );
            sleep(Duration::from_secs(5)).await;
            continue;
        }
        const MAX_PACKET_SIZE: u32 = 1024 * 1024; // 1 MB upper bound for usbmuxd plist packets
        let packet_size = u32::from_le_bytes(header[0..4].try_into().unwrap_or_default());
        if !(16..=MAX_PACKET_SIZE).contains(&packet_size) {
            eprintln!(
                "[ERR] Invalid handshake packet size {}. Retrying in 5s...",
                packet_size
            );
            sleep(Duration::from_secs(5)).await;
            continue;
        }
        let mut body = vec![0u8; (packet_size - 16) as usize];
        if let Err(e) = socket.read_exact(&mut body).await {
            eprintln!(
                "[ERR] Failed to read handshake body: {}. Retrying in 5s...",
                e
            );
            sleep(Duration::from_secs(5)).await;
            continue;
        }

        let handshake: plist::Dictionary = match plist::from_bytes(&body) {
            Ok(d) => d,
            Err(e) => {
                eprintln!(
                    "[ERR] Failed to parse handshake plist: {}. Retrying in 5s...",
                    e
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        match handshake
            .get("Number")
            .and_then(|x| x.as_unsigned_integer())
        {
            Some(0) => {}
            other => {
                eprintln!(
                    "[ERR] usbmuxd rejected Listen request: {:?}. Retrying in 5s...",
                    other
                );
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        }

        println!("[*] Connected to usbmuxd/netmuxd event stream");

        // Event loop
        loop {
            let mut header = [0u8; 16];
            if let Err(e) = socket.read_exact(&mut header).await {
                eprintln!("[WARN] usbmuxd stream disconnected: {}", e);
                break;
            }
            let packet_size = u32::from_le_bytes(header[0..4].try_into().unwrap_or_default());
            if !(16..=MAX_PACKET_SIZE).contains(&packet_size) {
                eprintln!("[WARN] Malformed usbmuxd packet size: {}", packet_size);
                break;
            }
            let mut body = vec![0u8; (packet_size - 16) as usize];
            if let Err(e) = socket.read_exact(&mut body).await {
                eprintln!("[WARN] Failed reading usbmuxd packet body: {}", e);
                break;
            }

            let msg: plist::Dictionary = match plist::from_bytes(&body) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[WARN] Failed parsing usbmuxd plist: {}", e);
                    continue;
                }
            };

            if let Some(msg_type) = msg.get("MessageType").and_then(|v| v.as_string()) {
                match msg_type {
                    "Attached" => {
                        let dev_id = msg
                            .get("DeviceID")
                            .and_then(|v| v.as_unsigned_integer())
                            .map(|v| v as u32);
                        let props = msg.get("Properties").and_then(|v| v.as_dictionary());
                        if let (Some(dev_id), Some(props)) = (dev_id, props) {
                            let udid = props
                                .get("SerialNumber")
                                .and_then(|v| v.as_string())
                                .unwrap_or("")
                                .to_string();
                            let conn_type_raw = props
                                .get("ConnectionType")
                                .and_then(|v| v.as_string())
                                .unwrap_or("Unknown");
                            let conn_str = match conn_type_raw {
                                "Network" => props
                                    .get("NetworkAddress")
                                    .and_then(|v| v.as_data())
                                    .and_then(parse_network_address)
                                    .map(|ip| format!("Network({})", ip))
                                    .unwrap_or_else(|| "Network".to_string()),
                                other => other.to_string(),
                            };

                            if !udid.is_empty() {
                                let _ = tx.send(DaemonEvent::Attached {
                                    dev_id,
                                    udid,
                                    conn_type: conn_str,
                                });
                            }
                        }
                    }
                    "Detached" => {
                        if let Some(dev_id) = msg
                            .get("DeviceID")
                            .and_then(|v| v.as_unsigned_integer())
                            .map(|v| v as u32)
                        {
                            let _ = tx.send(DaemonEvent::Detached { dev_id });
                        }
                    }
                    _ => {}
                }
            }
        }

        let _ = tx.send(DaemonEvent::MuxDisconnected);
        eprintln!("[WARN] usbmuxd stream ended. Reconnecting in 3s...");
        sleep(Duration::from_secs(3)).await;
    }
}
