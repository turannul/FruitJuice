use idevice::IdeviceService;
use idevice::provider::UsbmuxdProvider;
use idevice::services::companion_proxy::CompanionProxy;

pub fn mask_udid(udid: &str) -> &str {
    if udid.len() > 6 {
        &udid[udid.len() - 4..]
    } else {
        udid
    }
}

#[derive(Debug, Clone)]
pub struct CompanionDeviceData {
    pub udid: String,
    pub device_name: String,
    pub model_code: String,
    pub model_name: String,
    pub model_detail: String,
    pub hardware_model: String,
    pub product_version: String,
    pub serial: String,
    pub battery_cap: u8,
    pub is_charging: bool,
}

pub struct WatchModelInfo {
    pub model_name: &'static str,
    pub model_detail: &'static str,
}

const fn watch(model_name: &'static str, model_detail: &'static str) -> Option<WatchModelInfo> {
    Some(WatchModelInfo {
        model_name,
        model_detail,
    })
}

pub fn get_watch_info(code: &str) -> Option<WatchModelInfo> {
    match code.to_uppercase().as_str() {
        "WATCH1,1" => watch("Apple Watch (1st generation)", "38mm Case"),
        "WATCH1,2" => watch("Apple Watch (1st generation)", "42mm Case"),
        "WATCH2,6" => watch("Apple Watch Series 1", "38mm Case - Aluminum"),
        "WATCH2,7" => watch("Apple Watch Series 1", "42mm Case - Aluminum"),
        "WATCH2,3" => watch("Apple Watch Series 2", "38mm Case"),
        "WATCH2,4" => watch("Apple Watch Series 2", "42mm Case"),
        "WATCH3,1" => watch("Apple Watch Series 3 (GPS + Cellular)", "38mm Case"),
        "WATCH3,2" => watch("Apple Watch Series 3 (GPS + Cellular)", "42mm Case"),
        "WATCH3,3" => watch("Apple Watch Series 3 (GPS)", "38mm Case - Aluminum"),
        "WATCH3,4" => watch("Apple Watch Series 3 (GPS)", "42mm Case - Aluminum"),
        "WATCH4,1" => watch("Apple Watch Series 4 (GPS)", "40mm Case - Aluminum"),
        "WATCH4,2" => watch("Apple Watch Series 4 (GPS)", "44mm Case - Aluminum"),
        "WATCH4,3" => watch("Apple Watch Series 4 (GPS + Cellular)", "40mm Case"),
        "WATCH4,4" => watch("Apple Watch Series 4 (GPS + Cellular)", "44mm Case"),
        "WATCH5,1" => watch("Apple Watch Series 5 (GPS)", "40mm Case - Aluminum"),
        "WATCH5,2" => watch("Apple Watch Series 5 (GPS)", "44mm Case - Aluminum"),
        "WATCH5,3" => watch("Apple Watch Series 5 (GPS + Cellular)", "40mm Case"),
        "WATCH5,4" => watch("Apple Watch Series 5 (GPS + Cellular)", "44mm Case"),
        "WATCH5,9" => watch("Apple Watch SE (GPS)", "40mm Case - Aluminum"),
        "WATCH5,10" => watch("Apple Watch SE (GPS)", "44mm Case - Aluminum"),
        "WATCH5,11" => watch("Apple Watch SE (GPS + Cellular)", "40mm Case - Aluminum"),
        "WATCH5,12" => watch("Apple Watch SE (GPS + Cellular)", "44mm Case - Aluminum"),
        "WATCH6,1" => watch("Apple Watch Series 6 (GPS)", "40mm Case - Aluminum"),
        "WATCH6,2" => watch("Apple Watch Series 6 (GPS)", "44mm Case - Aluminum"),
        "WATCH6,3" => watch("Apple Watch Series 6 (GPS + Cellular)", "40mm Case"),
        "WATCH6,4" => watch("Apple Watch Series 6 (GPS + Cellular)", "44mm Case"),
        "WATCH6,6" => watch("Apple Watch Series 7 (GPS)", "41mm Case - Aluminum"),
        "WATCH6,7" => watch("Apple Watch Series 7 (GPS)", "45mm Case - Aluminum"),
        "WATCH6,8" => watch("Apple Watch Series 7 (GPS + Cellular)", "41mm Case"),
        "WATCH6,9" => watch("Apple Watch Series 7 (GPS + Cellular)", "45mm Case"),
        "WATCH6,10" => watch(
            "Apple Watch SE (2nd generation) (GPS)",
            "40mm Case - Aluminum",
        ),
        "WATCH6,11" => watch(
            "Apple Watch SE (2nd generation) (GPS)",
            "44mm Case - Aluminum",
        ),
        "WATCH6,12" => watch(
            "Apple Watch SE (2nd generation) (GPS + Cellular)",
            "40mm Case - Aluminum",
        ),
        "WATCH6,13" => watch(
            "Apple Watch SE (2nd generation) (GPS + Cellular)",
            "44mm Case - Aluminum",
        ),
        "WATCH6,14" => watch("Apple Watch Series 8 (GPS)", "41mm Case - Aluminum"),
        "WATCH6,15" => watch("Apple Watch Series 8 (GPS)", "45mm Case - Aluminum"),
        "WATCH6,16" => watch("Apple Watch Series 8 (GPS + Cellular)", "41mm Case"),
        "WATCH6,17" => watch("Apple Watch Series 8 (GPS + Cellular)", "45mm Case"),
        "WATCH6,18" => watch("Apple Watch Ultra", "49mm Case - Titanium"),
        "WATCH7,1" => watch("Apple Watch Series 9 (GPS)", "41mm Case - Aluminum"),
        "WATCH7,2" => watch("Apple Watch Series 9 (GPS)", "45mm Case - Aluminum"),
        "WATCH7,3" => watch("Apple Watch Series 9 (GPS + Cellular)", "41mm Case"),
        "WATCH7,4" => watch("Apple Watch Series 9 (GPS + Cellular)", "45mm Case"),
        "WATCH7,5" => watch("Apple Watch Ultra 2", "49mm Case - Titanium"),
        "WATCH7,8" => watch("Apple Watch Series 10 (GPS)", "42mm Case - Aluminum"),
        "WATCH7,9" => watch("Apple Watch Series 10 (GPS)", "46mm Case - Aluminum"),
        "WATCH7,10" => watch("Apple Watch Series 10 (GPS + Cellular)", "42mm Case"),
        "WATCH7,11" => watch("Apple Watch Series 10 (GPS + Cellular)", "46mm Case"),
        "WATCH7,12" => watch("Apple Watch Ultra 3", "49mm Case - Titanium"),
        "WATCH7,13" => watch(
            "Apple Watch SE (3rd generation) (GPS)",
            "40mm Case - Aluminum",
        ),
        "WATCH7,14" => watch(
            "Apple Watch SE (3rd generation) (GPS)",
            "44mm Case - Aluminum",
        ),
        "WATCH7,15" => watch(
            "Apple Watch SE (3rd generation) (GPS + Cellular)",
            "40mm Case - Aluminum",
        ),
        "WATCH7,16" => watch(
            "Apple Watch SE (3rd generation) (GPS + Cellular)",
            "44mm Case - Aluminum",
        ),
        "WATCH7,17" => watch("Apple Watch Series 11 (GPS)", "42mm Case - Aluminum"),
        "WATCH7,18" => watch("Apple Watch Series 11 (GPS)", "46mm Case - Aluminum"),
        "WATCH7,19" => watch("Apple Watch Series 11 (GPS + Cellular)", "42mm Case"),
        "WATCH7,20" => watch("Apple Watch Series 11 (GPS + Cellular)", "46mm Case"),
        _ => None,
    }
}

pub fn format_watch_model(code: &str, hw_model: &str) -> String {
    let clean_code = code.trim();
    if !clean_code.is_empty() {
        if let Some(info) = get_watch_info(clean_code) {
            return info.model_name.to_string();
        }
        return clean_code.to_string();
    }
    let clean_hw = hw_model.trim();
    if !clean_hw.is_empty() {
        return clean_hw.to_string();
    }
    "Apple Watch".to_string()
}

pub fn format_watch_detail(code: &str) -> String {
    get_watch_info(code)
        .map(|info| info.model_detail.to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

pub async fn query_companion_battery(
    provider: &UsbmuxdProvider,
    watch_udid: &str,
) -> Option<(u8, bool)> {
    let fetch = |key: &'static str| {
        let prov = provider.clone();
        let w_udid = watch_udid.to_string();
        async move {
            match CompanionProxy::connect(&prov).await {
                Ok(mut c) => c.get_value(&w_udid, key).await.ok(),
                Err(e) => {
                    eprintln!(
                        "[ERR] CompanionProxy connect failed for [...{}]: {}",
                        mask_udid(&prov.udid),
                        e
                    );
                    None
                }
            }
        }
    };

    let (cap_raw, chg_raw) =
        tokio::join!(fetch("BatteryCurrentCapacity"), fetch("BatteryIsCharging"));

    let cap = cap_raw.and_then(|v| v.as_unsigned_integer()).unwrap_or(0) as u8;
    let is_charging = chg_raw.and_then(|v| v.as_boolean()).unwrap_or(false);

    Some((cap, is_charging))
}

pub async fn query_all_companions(provider: &UsbmuxdProvider) -> Vec<CompanionDeviceData> {
    let mut result = Vec::new();

    let mut comp = match CompanionProxy::connect(provider).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[ERR] Failed to connect CompanionProxy for [...{}]: {}",
                mask_udid(&provider.udid),
                e
            );
            return result;
        }
    };

    let registry = match comp.get_device_registry().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "[ERR] Failed to get companion device registry for [...{}]: {}",
                mask_udid(&provider.udid),
                e
            );
            return result;
        }
    };

    for watch_udid in registry {
        let fetch_val = |key: &'static str| {
            let prov = provider.clone();
            let w_udid = watch_udid.clone();
            async move {
                match CompanionProxy::connect(&prov).await {
                    Ok(mut c) => c.get_value(&w_udid, key).await.ok(),
                    Err(e) => {
                        eprintln!(
                            "[ERR] CompanionProxy connect failed for [...{}]: {}",
                            mask_udid(&prov.udid),
                            e
                        );
                        None
                    }
                }
            }
        };

        let (name_raw, model_raw, hw_model_raw, ver_raw, serial_raw, cap_raw, chg_raw) = tokio::join!(
            fetch_val("DeviceName"),
            fetch_val("ProductType"),
            fetch_val("HardwareModel"),
            fetch_val("ProductVersion"),
            fetch_val("SerialNumber"),
            fetch_val("BatteryCurrentCapacity"),
            fetch_val("BatteryIsCharging"),
        );

        let name = name_raw
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_else(|| "Apple Watch".to_string());

        let model = model_raw
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_default();

        let hw_model = hw_model_raw
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_default();

        let version = ver_raw
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_default();

        let serial = serial_raw
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_default();

        let cap = cap_raw.and_then(|v| v.as_unsigned_integer()).unwrap_or(0) as u8;
        let charging = chg_raw.and_then(|v| v.as_boolean()).unwrap_or(false);

        let model_name = format_watch_model(&model, &hw_model);
        let model_detail = format_watch_detail(&model);

        result.push(CompanionDeviceData {
            udid: watch_udid,
            device_name: name,
            model_code: model,
            model_name,
            model_detail,
            hardware_model: hw_model,
            product_version: version,
            serial,
            battery_cap: cap,
            is_charging: charging,
        });
    }

    result
}
