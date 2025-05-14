use anyhow::Result;
use esp_idf_svc::sys::{
    hid_report_type_t, tinyusb_config_t, tinyusb_config_t__bindgen_ty_1,
    tinyusb_config_t__bindgen_ty_2, tinyusb_config_t__bindgen_ty_2__bindgen_ty_1,
    tinyusb_driver_install, tud_control_status, tud_vendor_n_read_flush, tud_vendor_n_write,
    tud_vendor_n_write_flush,
};
use esp_idf_svc::sys::{tud_control_xfer, tud_hid_n_report, tusb_control_request_t};
use std::collections::VecDeque;
use std::ptr;
use std::sync::{mpsc::Sender, LazyLock, Mutex, OnceLock};
use thiserror::Error;

use crate::bsp::usb_desc::{
    TUSB_DESC_BOS, TUSB_DESC_CONFIGURATION, TUSB_DESC_DEVICE, TUSB_DESC_HID_REPORT,
};
use crate::events::{AppEvent, UsbStatus};

const MAGIC_WORD: u32 = 0xE59DECC0;
// Magic Word + Payload length bytes
const HEADER_SIZE: usize = 8;
// Max payload length is 4KB
const MAX_PAYLOAD_LENGTH: usize = 4 * 1024;

#[derive(Debug, Clone, Error)]
pub enum UsbMessageError {
    #[error("Not enough space to send message")]
    NotEnoughSpace,
    #[error("Failed to frame message: {0}")]
    FailedToFrame(String),
}

#[derive(Debug, Clone, Default)]
struct UsbRxBuffer {
    data: VecDeque<u8>,
    state: UsbRxState,
}

#[derive(Debug, Clone, Default)]
enum UsbRxState {
    #[default]
    AwaitingMagicWord,
    ReadingLength,
    ReadingPayload {
        payload_length: u32,
    },
}

static USB_UPDATE_TX: OnceLock<Mutex<Sender<AppEvent>>> = OnceLock::new();
static PROCESS_MESSAGE_TX: OnceLock<Mutex<Sender<Vec<u8>>>> = OnceLock::new();
static USB_RX_BUFFER: LazyLock<Mutex<UsbRxBuffer>> =
    LazyLock::new(|| Mutex::new(UsbRxBuffer::default()));

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_hid_descriptor_report_cb(instance: u8) -> *const u8 {
    log::info!(
        "tud_hid_descriptor_report_cb called (instance={})",
        instance
    );
    TUSB_DESC_HID_REPORT.as_ptr()
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
    0
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

fn send_usb_update(status: UsbStatus) {
    let usb_update_tx = match USB_UPDATE_TX.get() {
        Some(tx) => match tx.lock() {
            Ok(tx) => tx,
            Err(e) => {
                log::error!("Failed to lock USB_UPDATE_TX: {}", e);
                return;
            }
        },
        None => {
            log::error!("USB_UPDATE_TX is not initialized");
            return;
        }
    };
    usb_update_tx.send(AppEvent::UsbUpdate(status)).unwrap();
}

fn process_message(message: Vec<u8>) -> bool {
    let usb_message_tx = match PROCESS_MESSAGE_TX.get() {
        Some(tx) => match tx.lock() {
            Ok(tx) => tx,
            Err(e) => {
                log::error!("Failed to lock PROCESS_MESSAGE_TX: {}", e);
                return false;
            }
        },
        None => {
            log::error!("PROCESS_MESSAGE_TX is not initialized");
            return false;
        }
    };
    usb_message_tx.send(message).unwrap();
    true
}

fn frame_message(payload_bytes: &[u8]) -> Result<Vec<u8>> {
    let payload_length = payload_bytes.len();

    if payload_length > MAX_PAYLOAD_LENGTH {
        return Err(UsbMessageError::FailedToFrame(format!(
            "Payload too large: {} bytes, max is {}",
            payload_length, MAX_PAYLOAD_LENGTH
        ))
        .into());
    }

    let mut frame = Vec::with_capacity(HEADER_SIZE + payload_length);
    frame.extend_from_slice(&MAGIC_WORD.to_le_bytes());
    frame.extend_from_slice(&(payload_length as u32).to_le_bytes()); // Ensure it's u32 for length field
    frame.extend_from_slice(payload_bytes);
    Ok(frame)
}

pub fn send_usb_message(message: Vec<u8>) -> Result<()> {
    // Ideally we should check if there is enough space to send the message using tud_vendor_n_write_available
    // but FIFO is not implemented in esp-tinyusb yet
    //TODO: Implement mutex
    let frame = frame_message(&message); // Pass as slice
    match frame {
        Ok(frame) => {
            unsafe {
                let mut total_bytes_written: usize = 0;
                while total_bytes_written < frame.len() {
                    let chunk_ptr = frame.as_ptr().add(total_bytes_written);
                    let remaining_bytes = frame.len() - total_bytes_written;
                    let bytes_written =
                        tud_vendor_n_write(0, chunk_ptr as *const _, remaining_bytes as u32);
                    log::info!(
                    "tud_vendor_n_write called with frame_len: {}. Bytes reportedly written to FIFO: {}",
                    frame.len(),
                    bytes_written
                );
                    tud_vendor_n_write_flush(0);
                    total_bytes_written += bytes_written as usize;
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_mount_cb() {
    log::info!("tud_mount_cb called");
    send_usb_update(UsbStatus::Connected);
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_unmount_cb() {
    log::info!("tud_unmount_cb called");
    send_usb_update(UsbStatus::Disconnected);
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_suspend_cb(do_remote_wakeup: bool) {
    log::info!(
        "tud_suspend_cb called (do_remote_wakeup={})",
        do_remote_wakeup
    );
    send_usb_update(UsbStatus::Suspended);
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_resume_cb() {
    log::info!("tud_resume_cb called");
    send_usb_update(UsbStatus::Connected);
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_vendor_rx_cb(itf: u8, buffer: *const u8, len: u16) {
    log::info!("tud_vendor_rx_cb called with {} bytes", len);

    let new_data_slice = unsafe { std::slice::from_raw_parts(buffer as *const u8, len as usize) };

    match USB_RX_BUFFER.lock() {
        Ok(mut buffer_guard) => {
            buffer_guard.data.extend(new_data_slice);
            log::trace!("Extended RX buffer, new size: {}", buffer_guard.data.len());
            // Process the buffer if we have at least magic word + payload length + some data
            if buffer_guard.data.len() > HEADER_SIZE {
                // Drop the guard to unlock the buffer before calling process_rx_buffer
                // because process_rx_buffer will lock the buffer again but has enough
                // granularity to unlock it when sending data over a channel to the main thread
                drop(buffer_guard);
                process_rx_buffer();
            }
        }
        Err(e) => {
            log::error!("Failed to lock USB_RX_BUFFER: {}", e);
        }
    }
    // Prepare to receive more data
    unsafe {
        tud_vendor_n_read_flush(itf);
    }
}

/// Process the RX buffer if we have enough data to process
/// Returns true if a full message was found and processed
fn process_rx_buffer() -> bool {
    let mut guard = match USB_RX_BUFFER.lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("Failed to lock USB_RX_BUFFER: {}", e);
            return false;
        }
    };
    let UsbRxBuffer { data, state } = &mut *guard;
    let mut message_processed = false;

    // Loop because we may receive multiple messages in the buffer concatenated together
    loop {
        match state {
            UsbRxState::AwaitingMagicWord => {
                if data.len() >= 4 {
                    // Peek at the first 4 bytes to look for the magic word
                    // without consuming them
                    let magic_word: u32 = ((data[0] as u32) << 24)
                        | ((data[1] as u32) << 16)
                        | ((data[2] as u32) << 8)
                        | (data[3] as u32);
                    // let magic_word = u32::from_le_bytes(magic_word_bytes);
                    if magic_word == MAGIC_WORD {
                        // Found the magic word, move the buffer forward by 4 bytes
                        data.drain(..4);
                        log::info!("Found magic word, moving to ReadingLength state");
                        *state = UsbRxState::ReadingLength;
                    } else {
                        log::warn!(
                            "Magic word mismatch. Expected {:x}, got {:x}. Discarding byte: {:#04x}",
                            MAGIC_WORD,
                            magic_word,
                            data[0]
                        );
                        data.pop_front();
                        if data.len() < 4 {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            UsbRxState::ReadingLength => {
                if data.len() >= 4 {
                    // Peek at the first 4 bytes to look for the payload length
                    // without consuming them
                    let payload_length: u32 = ((data[0] as u32) << 24)
                        | ((data[1] as u32) << 16)
                        | ((data[2] as u32) << 8)
                        | (data[3] as u32);
                    if payload_length > 0 && payload_length <= MAX_PAYLOAD_LENGTH as u32 {
                        log::info!(
                            "Payload length: {}. Moving to ReadingPayload state",
                            payload_length
                        );
                        data.drain(..4);
                        *state = UsbRxState::ReadingPayload { payload_length };
                    } else {
                        log::warn!(
                            "Invalid payload length: {}. Max is {}. Moving back to AwaitingMagicWord state",
                            payload_length,
                            MAX_PAYLOAD_LENGTH
                        );
                        *state = UsbRxState::AwaitingMagicWord;
                        // Don't break here because we want to continue processing the buffer
                    }
                } else {
                    break;
                }
            }
            UsbRxState::ReadingPayload { payload_length } => {
                let payload_length = *payload_length as usize;
                if data.len() >= payload_length {
                    let payload_bytes: Vec<u8> = data.drain(0..payload_length).collect();
                    log::info!(
                        "Received payload of length {}. Deserializing...",
                        payload_length
                    );
                    // Send over to protocol handler over a channel to deserialize and process
                    if process_message(payload_bytes) {
                        log::info!("Sent payload to protocol handler");
                        message_processed = true;
                    }

                    // Move back to AwaitingMagicWord state to process the next message
                    *state = UsbRxState::AwaitingMagicWord;
                } else {
                    break;
                }
            }
        }
    }
    drop(guard);
    message_processed
}

#[allow(unused_variables)]
#[no_mangle]
extern "C" fn tud_vendor_tx_cb(itf: u8, len: u16) {
    log::info!("tud_vendor_tx_cb called (itf={}, len={})", itf, len);
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
    TUSB_DESC_BOS.as_ptr()
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
                    let len = crate::bsp::usb_desc::TUSB_DESC_WEBUSB_URL.len() as u16;
                    let req_len = req.wLength;
                    let len_to_send = len.min(req_len);

                    log::info!(
                        "Sending WebUSB Descriptor (requesting {}, sending {})",
                        req_len,
                        len_to_send
                    );

                    unsafe { tud_control_xfer(rhport, request, url_ptr as *mut _, len_to_send) }
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

                        unsafe { tud_control_xfer(rhport, request, desc as *mut _, len_to_send) }
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

        crate::bsp::usb_desc::TURB_REQUEST_TYPE_CLASS => {
            log::info!("Class request received");
            if req.bRequest == 0x22 {
                log::info!("WebUSB interface connected");
            }
            unsafe { tud_control_status(rhport, request) }
        }

        _ => {
            log::info!("Unknown request type {}", request_type);
            false // Not handled
        }
    }
}

pub struct Usb;

impl Usb {
    #[allow(unused_unsafe)]
    #[allow(static_mut_refs)]
    pub fn new(usb_update_tx: Sender<AppEvent>, message_tx: Sender<Vec<u8>>) -> Self {
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

        match USB_UPDATE_TX.set(Mutex::new(usb_update_tx)) {
            Ok(_) => (),
            Err(e) => {
                log::error!("Failed to set USB_UPDATE_TX: {:?}", e);
            }
        }

        match PROCESS_MESSAGE_TX.set(Mutex::new(message_tx)) {
            Ok(_) => (),
            Err(e) => {
                log::error!("Failed to set USB_MESSAGE_TX: {:?}", e);
            }
        }

        send_usb_update(UsbStatus::Initialized);

        Self {}
    }

    pub fn send_hid_report<T>(itf: u8, report_id: u8, report_data: &T, report_len: usize) -> bool {
        unsafe {
            let data_ptr = report_data as *const _ as *const core::ffi::c_void;
            tud_hid_n_report(itf, report_id, data_ptr, report_len as u16)
        }
    }
}
