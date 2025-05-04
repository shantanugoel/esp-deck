use crate::{
    bsp::{
        usb::UsbHid,
        usb_desc::{
            ConsumerReport, KeyboardReport, MouseReport, REPORT_ID_CONSUMER, REPORT_ID_KEYBOARD,
            REPORT_ID_MOUSE,
        },
    },
    events::{AppEvent, UsbHidCommand},
};
use anyhow::Result;
use esp_idf_svc::sys::tud_hid_n_ready;
use std::mem::size_of;
use std::sync::mpsc::Receiver;

pub struct UsbHidClient;

impl UsbHidClient {
    pub fn run(command_rx: Receiver<AppEvent>) -> Result<()> {
        log::info!("Starting USB HID client");

        let _usb_hid = UsbHid::new();
        // The TinyUSB HID interface number. Since we configure only one HID
        // interface in sdkconfig (even if composite), this should be 0.
        const TUSB_HID_ITF: u8 = 0;

        loop {
            match command_rx.recv() {
                Ok(app_event) => {
                    match app_event {
                        AppEvent::UsbHidCommand(cmd) => {
                            // Check if the HID interface is ready
                            let mut attempts = 0;
                            let ready = loop {
                                if unsafe { tud_hid_n_ready(TUSB_HID_ITF) } {
                                    break true;
                                }
                                if attempts > 10 {
                                    log::error!("USB HID client: HID interface not ready after 10 attempts. Exiting");
                                    break false;
                                }
                                attempts += 1;
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            };

                            if !ready {
                                log::error!("USB HID client: HID interface not ready after 10 attempts. Exiting");
                                continue;
                            }

                            let report_sent = match cmd {
                                UsbHidCommand::SendKeyboard(report) => UsbHid::send_hid_report(
                                    TUSB_HID_ITF,
                                    REPORT_ID_KEYBOARD,
                                    &report,
                                    size_of::<KeyboardReport>(),
                                ),
                                UsbHidCommand::SendMouse(report) => UsbHid::send_hid_report(
                                    TUSB_HID_ITF,
                                    REPORT_ID_MOUSE,
                                    &report,
                                    size_of::<MouseReport>(),
                                ),
                                UsbHidCommand::SendConsumer(report) => UsbHid::send_hid_report(
                                    TUSB_HID_ITF,
                                    REPORT_ID_CONSUMER,
                                    &report,
                                    size_of::<ConsumerReport>(),
                                ),
                            };
                            if report_sent {
                                log::info!(
                                    "USB HID client: Report sent successfully for {:?}",
                                    cmd
                                );
                            } else {
                                log::error!("USB HID client: Failed to send report for {:?}", cmd);
                            }
                        }
                        _ => {
                            log::info!(
                                "USB HID client: Received event but not implemented: {:?}",
                                app_event
                            );
                        }
                    }
                }
                Err(_) => {
                    log::error!("USB HID client: Error receiving command. Exiting");
                    break;
                }
            }
        }
        Ok(())
    }
}
