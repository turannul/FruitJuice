use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::xml::{extract_xml_bool, extract_xml_int, extract_xml_string};

type IdeviceT = *mut std::ffi::c_void;
type CompanionProxyClientT = *mut std::ffi::c_void;
type PlistT = *mut std::ffi::c_void;

const IDEVICE_LOOKUP_USBMUX: i32 = 1 << 1;
const IDEVICE_LOOKUP_NETWORK: i32 = 1 << 2;

#[link(name = "imobiledevice-1.0")]
unsafe extern "C" {
    fn idevice_new_with_options(device: *mut IdeviceT, udid: *const c_char, options: i32) -> i32;
    fn idevice_free(device: IdeviceT) -> i32;
    fn companion_proxy_client_start_service(
        device: IdeviceT,
        client: *mut CompanionProxyClientT,
        label: *const c_char,
    ) -> i32;
    fn companion_proxy_client_free(client: CompanionProxyClientT) -> i32;
    fn companion_proxy_get_device_registry(
        client: CompanionProxyClientT,
        paired_devices: *mut PlistT,
    ) -> i32;
    fn companion_proxy_get_value_from_registry(
        client: CompanionProxyClientT,
        companion_udid: *const c_char,
        key: *const c_char,
        value: *mut PlistT,
    ) -> i32;
}

#[link(name = "plist-2.0")]
unsafe extern "C" {
    fn plist_to_xml(plist: PlistT, plist_xml: *mut *mut c_char, length: *mut u32);
    fn plist_free(plist: PlistT);
    fn plist_mem_free(ptr: *mut std::ffi::c_void);
}

unsafe fn plist_to_string(plist: PlistT) -> Option<String> {
    if plist.is_null() {
        return None;
    }
    let mut xml_ptr: *mut c_char = ptr::null_mut();
    let mut len: u32 = 0;
    unsafe {
        plist_to_xml(plist, &mut xml_ptr, &mut len);
        if xml_ptr.is_null() {
            return None;
        }
        let c_str = CStr::from_ptr(xml_ptr);
        let s = c_str.to_string_lossy().into_owned();
        plist_mem_free(xml_ptr as *mut std::ffi::c_void);
        Some(s)
    }
}

pub struct CompanionDeviceData {
    pub udid: String,
    pub device_name: String,
    pub model_code: String,
    pub serial: String,
    pub battery_cap: u8,
    pub is_charging: bool,
}

pub fn query_companion_battery(parent_udid: &str, watch_udid: &str) -> Option<(u8, bool)> {
    let c_parent = CString::new(parent_udid).ok()?;
    let c_watch = CString::new(watch_udid).ok()?;
    let c_label = CString::new("fruitjuiced").ok()?;
    let c_cap_key = CString::new("BatteryCurrentCapacity").ok()?;
    let c_chg_key = CString::new("BatteryIsCharging").ok()?;

    unsafe {
        let mut dev: IdeviceT = ptr::null_mut();
        if idevice_new_with_options(
            &mut dev,
            c_parent.as_ptr(),
            IDEVICE_LOOKUP_USBMUX | IDEVICE_LOOKUP_NETWORK,
        ) != 0
            || dev.is_null()
        {
            return None;
        }

        let mut client: CompanionProxyClientT = ptr::null_mut();
        if companion_proxy_client_start_service(dev, &mut client, c_label.as_ptr()) != 0
            || client.is_null()
        {
            idevice_free(dev);
            return None;
        }

        let mut cap_plist: PlistT = ptr::null_mut();
        let mut cap = 0u8;
        if companion_proxy_get_value_from_registry(
            client,
            c_watch.as_ptr(),
            c_cap_key.as_ptr(),
            &mut cap_plist,
        ) == 0
            && !cap_plist.is_null()
        {
            if let Some(xml) = plist_to_string(cap_plist) {
                cap = extract_xml_int(&xml, "BatteryCurrentCapacity") as u8;
            }
            plist_free(cap_plist);
        }
        companion_proxy_client_free(client);

        let mut client2: CompanionProxyClientT = ptr::null_mut();
        let mut is_charging = false;
        if companion_proxy_client_start_service(dev, &mut client2, c_label.as_ptr()) == 0
            && !client2.is_null()
        {
            let mut chg_plist: PlistT = ptr::null_mut();
            if companion_proxy_get_value_from_registry(
                client2,
                c_watch.as_ptr(),
                c_chg_key.as_ptr(),
                &mut chg_plist,
            ) == 0
                && !chg_plist.is_null()
            {
                if let Some(xml) = plist_to_string(chg_plist) {
                    is_charging = extract_xml_bool(&xml, "BatteryIsCharging");
                }
                plist_free(chg_plist);
            }
            companion_proxy_client_free(client2);
        }

        idevice_free(dev);
        Some((cap, is_charging))
    }
}

pub fn query_all_companions(parent_udid: &str) -> Vec<CompanionDeviceData> {
    let mut result = Vec::new();
    let c_parent = match CString::new(parent_udid) {
        Ok(s) => s,
        Err(_) => return result,
    };
    let c_label = CString::new("fruitjuiced").unwrap();

    unsafe {
        let mut dev: IdeviceT = ptr::null_mut();
        if idevice_new_with_options(
            &mut dev,
            c_parent.as_ptr(),
            IDEVICE_LOOKUP_USBMUX | IDEVICE_LOOKUP_NETWORK,
        ) != 0
            || dev.is_null()
        {
            return result;
        }

        let mut client: CompanionProxyClientT = ptr::null_mut();
        if companion_proxy_client_start_service(dev, &mut client, c_label.as_ptr()) != 0
            || client.is_null()
        {
            idevice_free(dev);
            return result;
        }

        let mut reg_plist: PlistT = ptr::null_mut();
        let mut companion_udids = Vec::new();
        if companion_proxy_get_device_registry(client, &mut reg_plist) == 0 && !reg_plist.is_null()
        {
            if let Some(xml) = plist_to_string(reg_plist) {
                let mut cursor = &xml[..];
                while let Some(start) = cursor.find("<string>") {
                    let after_start = &cursor[start + 8..];
                    if let Some(end) = after_start.find("</string>") {
                        let udid = after_start[..end].trim().to_string();
                        if !udid.is_empty() {
                            companion_udids.push(udid);
                        }
                        cursor = &after_start[end + 9..];
                    } else {
                        break;
                    }
                }
            }
            plist_free(reg_plist);
        }
        companion_proxy_client_free(client);

        for c_udid in companion_udids {
            let c_watch = CString::new(c_udid.as_str()).unwrap();
            let fetch_key = |key_str: &str| -> Option<String> {
                let mut cl: CompanionProxyClientT = ptr::null_mut();
                if companion_proxy_client_start_service(dev, &mut cl, c_label.as_ptr()) != 0
                    || cl.is_null()
                {
                    return None;
                }
                let c_k = CString::new(key_str).unwrap();
                let mut v_plist: PlistT = ptr::null_mut();
                let mut text = None;
                if companion_proxy_get_value_from_registry(
                    cl,
                    c_watch.as_ptr(),
                    c_k.as_ptr(),
                    &mut v_plist,
                ) == 0
                    && !v_plist.is_null()
                {
                    text = plist_to_string(v_plist);
                    plist_free(v_plist);
                }
                companion_proxy_client_free(cl);
                text
            };

            let name = fetch_key("DeviceName")
                .map(|x| extract_xml_string(&x, "DeviceName"))
                .unwrap_or_else(|| "Apple Watch".to_string());
            let model = fetch_key("ProductType")
                .map(|x| extract_xml_string(&x, "ProductType"))
                .unwrap_or_default();
            let serial = fetch_key("SerialNumber")
                .map(|x| extract_xml_string(&x, "SerialNumber"))
                .unwrap_or_default();
            let cap = fetch_key("BatteryCurrentCapacity")
                .map(|x| extract_xml_int(&x, "BatteryCurrentCapacity") as u8)
                .unwrap_or(0);
            let charging = fetch_key("BatteryIsCharging")
                .map(|x| extract_xml_bool(&x, "BatteryIsCharging"))
                .unwrap_or(false);

            result.push(CompanionDeviceData {
                udid: c_udid,
                device_name: name,
                model_code: model,
                serial,
                battery_cap: cap,
                is_charging: charging,
            });
        }

        idevice_free(dev);
    }

    result
}
