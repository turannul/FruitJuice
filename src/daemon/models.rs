use std::process::Command;

pub fn convert_model_readable(code: &str) -> String {
    let code_upper = code.to_uppercase();
    match code_upper.as_str() {
        "V63AP" | "IPHONE19,1" => "iPhone 18 Pro".to_string(),
        "V64AP" | "IPHONE19,2" => "iPhone 18 Pro Max".to_string(),
        "V68AP" | "IPHONE19,3" | "IPHONE19,4" | "IPHONE19,5" => "iPhone Duo".to_string(),
        "D23AP" => "iPhone Air".to_string(),
        "V53AP" => "iPhone 17 Pro".to_string(),
        "V54AP" => "iPhone 17 Pro Max".to_string(),
        "V57AP" => "iPhone 17".to_string(),
        "V159AP" => "iPhone 17e".to_string(),
        "D93AP" | "IPHONE17,1" => "iPhone 16 Pro".to_string(),
        "D94AP" | "IPHONE17,2" => "iPhone 16 Pro Max".to_string(),
        "D42AP" | "IPHONE17,3" => "iPhone 16".to_string(),
        "D43AP" | "IPHONE17,4" => "iPhone 16 Plus".to_string(),
        "D83AP" | "IPHONE16,1" => "iPhone 15 Pro".to_string(),
        "D84AP" | "IPHONE16,2" => "iPhone 15 Pro Max".to_string(),
        "D37AP" | "IPHONE15,4" => "iPhone 15".to_string(),
        "D38AP" | "IPHONE15,5" => "iPhone 15 Plus".to_string(),
        "D73AP" | "IPHONE15,2" => "iPhone 14 Pro".to_string(),
        "D74AP" | "IPHONE15,3" => "iPhone 14 Pro Max".to_string(),
        "D27AP" | "IPHONE14,7" => "iPhone 14".to_string(),
        "D28AP" | "IPHONE14,8" => "iPhone 14 Plus".to_string(),
        "D63AP" | "IPHONE14,2" => "iPhone 13 Pro".to_string(),
        "D64AP" | "IPHONE14,3" => "iPhone 13 Pro Max".to_string(),
        "D16AP" | "IPHONE14,4" => "iPhone 13 mini".to_string(),
        "D17AP" | "IPHONE14,5" => "iPhone 13".to_string(),
        "D53PAP" | "D53P" | "IPHONE13,3" => "iPhone 12 Pro".to_string(),
        "D54PAP" | "D54P" | "IPHONE13,4" => "iPhone 12 Pro Max".to_string(),
        "D52GAP" | "IPHONE13,1" => "iPhone 12 mini".to_string(),
        "D53GAP" | "IPHONE13,2" => "iPhone 12".to_string(),
        "N104AP" | "IPHONE12,1" => "iPhone 11".to_string(),
        "D421AP" | "IPHONE12,3" => "iPhone 11 Pro".to_string(),
        "D431AP" | "IPHONE12,5" => "iPhone 11 Pro Max".to_string(),
        "D22AP" | "D221AP" | "IPHONE10,3" | "IPHONE10,6" => "iPhone X".to_string(),
        "D321AP" | "IPHONE11,2" => "iPhone XS".to_string(),
        "D331PAP" | "D331AP" | "IPHONE11,4" | "IPHONE11,6" => "iPhone XS Max".to_string(),
        "N841AP" | "IPHONE11,8" => "iPhone XR".to_string(),
        "N69AP" | "N69UAP" | "IPHONE8,4" => "iPhone SE (1st Gen)".to_string(),
        "D79AP" | "IPHONE12,8" => "iPhone SE (2nd Gen)".to_string(),
        "D49AP" | "IPHONE14,6" => "iPhone SE (3rd Gen)".to_string(),
        "J817" | "J818" | "J817AP" | "J818AP" => "iPad Pro 11-inch (M4)".to_string(),
        "J820" | "J821" | "J820AP" | "J821AP" => "iPad Pro 13-inch (M4)".to_string(),
        "J617" | "J618" | "J617AP" | "J618AP" => "iPad Pro 11-inch (4th Gen)".to_string(),
        "J620" | "J621" | "J620AP" | "J621AP" => "iPad Pro 12.9-inch (6th Gen)".to_string(),
        "J517" | "J518" | "J517AP" | "J518AP" => "iPad Pro 11-inch (3rd Gen)".to_string(),
        "J522" | "J523" | "J522AP" | "J523AP" => "iPad Pro 12.9-inch (5th Gen)".to_string(),
        "WATCH1,1" => "Apple Watch 38mm (1st Gen)".to_string(),
        "WATCH1,2" => "Apple Watch 42mm (1st Gen)".to_string(),
        "WATCH2,6" => "Apple Watch Series 1 38mm".to_string(),
        "WATCH2,7" => "Apple Watch Series 1 42mm".to_string(),
        "WATCH2,3" => "Apple Watch Series 2 38mm".to_string(),
        "WATCH2,4" => "Apple Watch Series 2 42mm".to_string(),
        "WATCH3,1" | "WATCH3,3" => "Apple Watch Series 3 38mm".to_string(),
        "WATCH3,2" | "WATCH3,4" => "Apple Watch Series 3 42mm".to_string(),
        "WATCH4,1" | "WATCH4,3" => "Apple Watch Series 4 40mm".to_string(),
        "WATCH4,2" | "WATCH4,4" => "Apple Watch Series 4 44mm".to_string(),
        "WATCH5,1" | "WATCH5,3" => "Apple Watch Series 5 40mm".to_string(),
        "WATCH5,2" | "WATCH5,4" => "Apple Watch Series 5 44mm".to_string(),
        "WATCH5,9" | "WATCH5,11" => "Apple Watch SE 40mm (1st Gen)".to_string(),
        "WATCH5,10" | "WATCH5,12" => "Apple Watch SE 44mm (1st Gen)".to_string(),
        "WATCH6,1" | "WATCH6,3" => "Apple Watch Series 6 40mm".to_string(),
        "WATCH6,2" | "WATCH6,4" => "Apple Watch Series 6 44mm".to_string(),
        "WATCH6,6" | "WATCH6,8" => "Apple Watch Series 7 41mm".to_string(),
        "WATCH6,7" | "WATCH6,9" => "Apple Watch Series 7 45mm".to_string(),
        "WATCH6,10" | "WATCH6,12" => "Apple Watch SE 40mm (2nd Gen)".to_string(),
        "WATCH6,11" | "WATCH6,13" => "Apple Watch SE 44mm (2nd Gen)".to_string(),
        "WATCH6,14" | "WATCH6,16" => "Apple Watch Series 8 41mm".to_string(),
        "WATCH6,15" | "WATCH6,17" => "Apple Watch Series 8 45mm".to_string(),
        "WATCH6,18" => "Apple Watch Ultra 49mm".to_string(),
        "WATCH7,1" | "WATCH7,3" => "Apple Watch Series 9 41mm".to_string(),
        "WATCH7,2" | "WATCH7,4" => "Apple Watch Series 9 45mm".to_string(),
        "WATCH7,5" => "Apple Watch Ultra 2 49mm".to_string(),
        "WATCH7,8" | "WATCH7,10" => "Apple Watch Series 10 42mm".to_string(),
        "WATCH7,9" | "WATCH7,11" => "Apple Watch Series 10 46mm".to_string(),
        _ => {
            if code.is_empty() {
                "Apple Device".to_string()
            } else if code_upper.starts_with("WATCH") {
                format!("Apple Watch ({})", code)
            } else {
                code.to_string()
            }
        }
    }
}

pub fn query_model_info(udid: &str, is_network: bool) -> (String, String, String) {
    let mut dev_name = "iPhone".to_string();
    let mut model_code = String::new();

    let mut id_cmd = Command::new("idevice_id");
    id_cmd.arg(udid);
    if let Ok(out) = id_cmd.output()
        && out.status.success()
    {
        let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !name.is_empty() {
            dev_name = name;
        }
    }

    let mut info_cmd = Command::new("ideviceinfo");
    if is_network {
        info_cmd.arg("--network");
    }
    info_cmd.args(["-u", udid, "-k", "HardwareModel"]);
    if let Ok(out) = info_cmd.output()
        && out.status.success()
    {
        model_code = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }
    if model_code.is_empty() {
        let mut pt_cmd = Command::new("ideviceinfo");
        if is_network {
            pt_cmd.arg("--network");
        }
        pt_cmd.args(["-u", udid, "-k", "ProductType"]);
        if let Ok(out) = pt_cmd.output()
            && out.status.success()
        {
            model_code = String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }

    let readable = convert_model_readable(&model_code);
    let class = if readable.contains("iPad") {
        "iPad".to_string()
    } else if readable.contains("Watch") {
        "Watch".to_string()
    } else {
        "iPhone".to_string()
    };

    let display_name = if dev_name == "iPhone" || dev_name == "iPad" || dev_name == "Apple Watch" {
        readable.clone()
    } else {
        dev_name
    };

    (display_name, readable, class)
}
