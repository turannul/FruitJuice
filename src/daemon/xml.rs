pub fn extract_xml_int(xml: &str, key: &str) -> i32 {
    let key_tag = format!("<key>{}</key>", key);
    if let Some(pos) = xml.find(&key_tag) {
        let rest = &xml[pos + key_tag.len()..];
        for tag in &["<integer>", "<real>"] {
            if let Some(val_start) = rest.find(tag) {
                let val_begin = val_start + tag.len();
                let end_tag = format!("</{}", &tag[1..]);
                if let Some(val_end) = rest[val_begin..].find(&end_tag) {
                    let val: f32 = rest[val_begin..val_begin + val_end]
                        .trim()
                        .parse()
                        .unwrap_or(0.0);
                    return val as i32;
                }
            }
        }
    }
    0
}

pub fn extract_xml_string(xml: &str, key: &str) -> String {
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

pub fn extract_xml_bool(xml: &str, key: &str) -> bool {
    let key_tag = format!("<key>{}</key>", key);
    let mut search_pos = 0;
    while let Some(pos) = xml[search_pos..].find(&key_tag) {
        let abs_pos = search_pos + pos + key_tag.len();
        let rest = xml[abs_pos..].trim_start();
        if rest.starts_with("<true/>") || rest.starts_with("<integer>1</integer>") {
            return true;
        }
        search_pos = abs_pos;
    }
    false
}
