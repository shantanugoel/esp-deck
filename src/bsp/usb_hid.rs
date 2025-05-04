use esp_idf_svc::sys::{hid_report_type_t, tud_mounted};
use esp_idf_svc::sys::{
    tinyusb_config_t, tinyusb_config_t__bindgen_ty_1, tinyusb_config_t__bindgen_ty_2,
    tinyusb_config_t__bindgen_ty_2__bindgen_ty_1, tinyusb_driver_install,
    tud_hid_n_keyboard_report,
};
use std::ffi::{c_char, CString};
use std::ptr;

use crate::bsp::usb_desc::{TUSB_DESC_CONFIGURATION, TUSB_DESC_DEVICE, TUSB_DESC_HID_REPORT};

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_hid_descriptor_report_cb(instance: u8) -> *const u8 {
    return TUSB_DESC_HID_REPORT.as_ptr();
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_hid_get_report_cb(
    instance: u8,
    report_id: u8,
    report_type: hid_report_type_t,
    buffer: *const u8,
    reqlen: u16,
) -> u16 {
    return 0;
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_hid_set_report_cb(
    instance: u8,
    report_id: u8,
    report_type: hid_report_type_t,
    buffer: *const u8,
    buffsize: u16,
) {
}

#[allow(dead_code)]
fn get_hid_string_descriptor() -> (*mut *const c_char, usize) {
    let strings = vec![
        CString::new(String::from_utf8(vec![0x09, 0x04]).unwrap()).unwrap(),
        CString::new("Shaan Labs Inc.").unwrap(),
        CString::new("ESP DECK").unwrap(),
        CString::new("42069").unwrap(),
        CString::new("ESP DECK HID Interface").unwrap(),
    ];

    let mut raw_pointers: Vec<*const c_char> = strings.iter().map(|c_str| c_str.as_ptr()).collect();
    let raw_ptr = raw_pointers.as_mut_ptr();
    (raw_ptr, raw_pointers.len())
}

pub struct UsbHid;

impl UsbHid {
    #[allow(unused_unsafe)]
    pub fn new() -> Self {
        let (string_descriptor, string_descriptor_count) = get_hid_string_descriptor();
        let tusb_config = tinyusb_config_t {
            string_descriptor: string_descriptor,
            string_descriptor_count: string_descriptor_count as i32,
            external_phy: false,
            self_powered: false,
            vbus_monitor_io: 0,
            __bindgen_anon_1: unsafe {
                tinyusb_config_t__bindgen_ty_1 {
                    device_descriptor: ptr::from_ref(&TUSB_DESC_DEVICE),
                }
            },
            __bindgen_anon_2: unsafe {
                tinyusb_config_t__bindgen_ty_2 {
                    __bindgen_anon_1: tinyusb_config_t__bindgen_ty_2__bindgen_ty_1 {
                        configuration_descriptor: TUSB_DESC_CONFIGURATION.as_ptr(),
                    },
                }
            },
        };

        unsafe { tinyusb_driver_install(&tusb_config) };

        Self {}
    }

    pub fn send_key(key: u8) {
        unsafe {
            if tud_mounted() {
                tud_hid_n_keyboard_report(0, 1, 0, [key, 0, 0, 0, 0, 0].as_mut_ptr());
                std::thread::sleep(std::time::Duration::from_millis(50));
                tud_hid_n_keyboard_report(0, 1, 0, ptr::null_mut());
            } else {
                log::info!("USB not mounted while trying to send key {}", key);
            }
        }
    }
}
