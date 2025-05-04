use esp_idf_svc::sys::hid_report_type_t;
use esp_idf_svc::sys::{
    tinyusb_config_t, tinyusb_config_t__bindgen_ty_1, tinyusb_config_t__bindgen_ty_2,
    tinyusb_config_t__bindgen_ty_2__bindgen_ty_1, tinyusb_driver_install, tud_control_xfer,
    tud_hid_n_report, tusb_control_request_t,
};
use std::ffi::{c_char, CString};
use std::ptr;

use crate::bsp::usb_desc::{
    TUSB_DESC_BOS, TUSB_DESC_CONFIGURATION, TUSB_DESC_DEVICE, TUSB_DESC_HID_REPORT,
};

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_hid_descriptor_report_cb(instance: u8) -> *const u8 {
    log::info!(
        "tud_hid_descriptor_report_cb called (instance={})",
        instance
    );
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
    log::info!(
        "tud_hid_get_report_cb called (instance={}, report_id={}, report_type={}, reqlen={})",
        instance,
        report_id,
        report_type,
        reqlen
    );
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
    log::info!(
        "tud_hid_set_report_cb called (instance={}, report_id={}, report_type={}, buffsize={})",
        instance,
        report_id,
        report_type,
        buffsize
    );
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_descriptor_bos_cb() -> *const u8 {
    log::info!("tud_descriptor_bos_cb called");
    return TUSB_DESC_BOS.as_ptr();
}

#[no_mangle]
pub extern "C" fn tud_descriptor_webusb_url_cb() -> *const u8 {
    // Return pointer to the URL descriptor defined in hid_desc.rs
    log::info!("tud_descriptor_webusb_url_cb called");
    crate::bsp::usb_desc::TUSB_DESC_WEBUSB_URL.as_ptr()
}

#[no_mangle]
pub extern "C" fn tud_vendor_control_xfer_cb(
    rhport: u8,
    request: *const tusb_control_request_t,
) -> bool {
    log::info!("tud_vendor_control_request_cb called (rhport={})", rhport);
    // Safety: Ensure request pointer is valid before dereferencing
    let req = unsafe { &*request };

    // Check if it's a WebUSB or MS OS 2.0 vendor request
    match req.bRequest {
        crate::bsp::usb_desc::VENDOR_REQUEST_WEBUSB => {
            // Handle WebUSB vendor requests (like getting landing page URL)
            // `tud_descriptor_webusb_url_cb` handles the URL descriptor specifically.
            // This might be used for other standard WebUSB requests if needed.
            unsafe {
                log::info!(
                    "WebUSB vendor request received: bmRequestType={}, bRequest={}",
                    req.__bindgen_anon_1.bmRequestType,
                    req.bRequest
                );
            }
            // Let TinyUSB handle standard WebUSB requests for URL descriptor
            false // Indicate not handled here, let TinyUSB proceed
        }

        crate::bsp::usb_desc::VENDOR_REQUEST_MICROSOFT => {
            // Handle Microsoft OS 2.0 descriptor requests
            if req.wIndex == 7 {
                // MS_OS_20_DESCRIPTOR_INDEX == 7
                log::info!("MS OS 2.0 Descriptor request received");
                let desc = crate::bsp::usb_desc::TUSB_DESC_MS_OS_20.as_ptr();
                let total_len = crate::bsp::usb_desc::TUSB_DESC_MS_OS_20.len() as u16;
                let req_len = req.wLength;
                let len_to_send = total_len.min(req_len);

                log::info!(
                    "Sending MS OS 2.0 Descriptor (requesting {}, sending {})",
                    req_len,
                    len_to_send
                );

                return unsafe { tud_control_xfer(rhport, request, desc as *mut _, len_to_send) };
            } else {
                log::info!("Unhandled MS OS 2.0 request");
                false // Not handled
            }
        }

        _ => {
            // Unknown vendor request
            log::info!("Unknown vendor request: {}", req.bRequest);
            false // Not handled
        }
    }
}

#[allow(dead_code)]
fn get_string_descriptor() -> (*mut *const c_char, usize) {
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

pub struct Usb;

impl Usb {
    #[allow(unused_unsafe)]
    pub fn new() -> Self {
        let (string_descriptor, string_descriptor_count) = get_string_descriptor();
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

    pub fn send_hid_report<T>(itf: u8, report_id: u8, report_data: &T, report_len: usize) -> bool {
        unsafe {
            let data_ptr = report_data as *const _ as *const core::ffi::c_void;
            tud_hid_n_report(itf, report_id, data_ptr, report_len as u16)
        }
    }
}
