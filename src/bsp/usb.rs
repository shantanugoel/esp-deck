use esp_idf_svc::sys::{
    hid_report_type_t, tinyusb_config_t, tinyusb_config_t__bindgen_ty_1,
    tinyusb_config_t__bindgen_ty_2, tinyusb_config_t__bindgen_ty_2__bindgen_ty_1,
    tinyusb_driver_install, tud_control_status,
};
use esp_idf_svc::sys::{tud_control_xfer, tud_hid_n_report, tusb_control_request_t};
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

// These are commented out because they are defined by esp_tinyusb alread
// If/When we are able to move away from esp_tinyusb, we can uncomment these
// #[allow(unused_variables)]
// #[no_mangle]
// extern "C" fn tud_descriptor_device_cb() -> *const u8 {
//     log::info!("tud_descriptor_device_cb called");
//     return TUSB_DESC_DEVICE.as_ptr();
// }

// #[allow(unused_variables)]
// #[no_mangle]
// extern "C" fn tud_descriptor_configuration_cb() -> *const u8 {
//     log::info!("tud_descriptor_configuration_cb called");
//     return TUSB_DESC_CONFIGURATION.as_ptr();
// }

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
    stage: u8,
    request: *const tusb_control_request_t,
) -> bool {
    log::info!(
        "tud_vendor_control_request_cb called (rhport={}, stage={})",
        rhport,
        stage
    );
    if stage != 1 {
        log::info!("stage is not CONTROL_STAGE_SETUP");
        return true;
    }
    // Safety: Ensure request pointer is valid before dereferencing
    let req = unsafe { &*request };

    // Check if it's a WebUSB or MS OS 2.0 vendor request
    let request_type = unsafe { req.__bindgen_anon_1.bmRequestType_bit.type_() };
    log::info!("request_type: {}", request_type);
    match request_type {
        crate::bsp::usb_desc::TURB_REQUEST_TYPE_VENDOR => {
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
                    let url_ptr = crate::bsp::usb_desc::TUSB_DESC_WEBUSB_URL.as_ptr();

                    return unsafe {
                        tud_control_xfer(
                            rhport,
                            request,
                            url_ptr as *mut _,
                            crate::bsp::usb_desc::TUSB_DESC_WEBUSB_URL.len() as u16,
                        )
                    };
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

                        return unsafe {
                            tud_control_xfer(rhport, request, desc as *mut _, len_to_send)
                        };
                    } else {
                        log::info!("Unhandled MS OS 2.0 request");
                        return false; // Not handled
                    }
                }

                _ => {
                    // Unknown vendor request
                    log::info!("Unknown vendor request: {}", req.bRequest);
                    return false; // Not handled
                }
            }
        }

        crate::bsp::usb_desc::TURB_REQUEST_TYPE_CLASS => {
            log::info!("Class request received");
            if req.bRequest == 0x22 {
                log::info!("WebUSB interface connected");
            }
            return unsafe { tud_control_status(rhport, request) };
        }

        _ => {
            log::info!("Unknown request type {}", request_type);
            return false; // Not handled
        }
    }
}

pub struct Usb;

impl Usb {
    #[allow(unused_unsafe)]
    #[allow(static_mut_refs)]
    pub fn new() -> Self {
        let tusb_config = tinyusb_config_t {
            string_descriptor: unsafe { crate::bsp::usb_desc::STRING_DESCRIPTOR.as_mut_ptr() },
            string_descriptor_count: crate::bsp::usb_desc::STRING_DESCRIPTOR_LEN as i32,
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
