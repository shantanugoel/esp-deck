use crate::bsp::hid_desc::{ConsumerReport, KeyboardReport, MouseReport};
use crate::events::{
    AppEvent,
    UsbHidCommand::{SendConsumer, SendKeyboard, SendMouse},
};
use keycode::{KeyMap, KeyMappingId};
use std::sync::mpsc::{Receiver, Sender};

pub struct Actor {
    actor_rx: Receiver<AppEvent>,
    usb_hid_tx: Sender<AppEvent>,
}

impl Actor {
    pub fn new(actor_rx: Receiver<AppEvent>, usb_hid_tx: Sender<AppEvent>) -> Self {
        Self {
            actor_rx,
            usb_hid_tx,
        }
    }

    pub fn run(&self) {
        log::info!("Starting Actor");
        loop {
            match self.actor_rx.recv() {
                Ok(app_event) => {
                    if let AppEvent::ButtonPressed(button_id) = app_event {
                        log::info!("Actor received ButtonPressed: {}", button_id);
                        match button_id {
                            1 => {
                                // Send Caps Lock press
                                let mut report = KeyboardReport {
                                    keys: [
                                        KeyMap::from(KeyMappingId::CapsLock).usb as u8,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0,
                                    ], // Keycode for Caps Lock
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendKeyboard(report)));
                                std::thread::sleep(std::time::Duration::from_millis(10));
                                // Send key release
                                report = KeyboardReport {
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendKeyboard(report)));
                            }
                            2 => {
                                // Make a square with mouse moves
                                let mut report = MouseReport {
                                    x: 0,
                                    y: 10,
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendMouse(report)));
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                // Send move back (or stop)
                                report = MouseReport {
                                    x: 10,
                                    y: 0,
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendMouse(report)));
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                // Send move back (or stop)
                                report = MouseReport {
                                    x: 0,
                                    y: -10,
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendMouse(report)));
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                // Send move back (or stop)
                                report = MouseReport {
                                    x: -10,
                                    y: 0,
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendMouse(report)));
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            _ => {
                                // Send Volume command (was Mute 0xe2, then generic Volume 0xe0)
                                // Keeping 0xe0 for now based on previous edits, but ideally this should be VolumeUp/Down
                                // or use the absolute value logic we discussed.
                                let mut report = ConsumerReport {
                                    usage: 0xe0,
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendConsumer(report)));
                                std::thread::sleep(std::time::Duration::from_millis(10));
                                // Send release
                                report = ConsumerReport {
                                    ..Default::default()
                                };
                                let _ = self
                                    .usb_hid_tx
                                    .send(AppEvent::UsbHidCommand(SendConsumer(report)));
                            }
                        }
                    } else {
                        log::warn!("Actor received unexpected event: {:?}", app_event);
                    }
                }
                Err(e) => {
                    log::error!("Actor failed to receive event: {}. Exiting.", e);
                    break; // Exit loop on channel error
                }
            }
        }
    }
}
