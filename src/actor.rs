use crate::bsp::usb_desc::{ConsumerReport, KeyboardReport, MouseReport};
use crate::events::{
    AppEvent, HidAction,
    UsbHidCommand::{SendConsumer, SendKeyboard, SendMouse},
};
use crate::mapper::Mapper;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct Actor {
    actor_rx: Receiver<AppEvent>,
    usb_hid_tx: Sender<AppEvent>,
    mapper: Mapper,
}

impl Actor {
    pub fn new(actor_rx: Receiver<AppEvent>, usb_hid_tx: Sender<AppEvent>, mapper: Mapper) -> Self {
        Self {
            actor_rx,
            usb_hid_tx,
            mapper,
        }
    }

    pub fn run(&mut self) {
        log::info!("Starting Actor");
        let mut current_mouse_report = MouseReport::default();

        loop {
            match self.actor_rx.recv() {
                Ok(app_event) => {
                    if let AppEvent::ButtonPressed(button_id) = app_event {
                        log::info!("Actor received ButtonPressed: {}", button_id);
                        let action_sequence = self.mapper.get_action_sequence(button_id);
                        log::info!("Actor received action sequence: {:?}", action_sequence);
                        for action in action_sequence {
                            log::debug!("Actor executing action: {:?}", action);
                            match action {
                                HidAction::KeyPress(modifier_bits, keycodes) => {
                                    let report = KeyboardReport {
                                        modifier: modifier_bits,
                                        keys: keycodes,
                                        ..Default::default()
                                    };
                                    log::debug!(
                                        "Actor sending KeyboardReport: modifier={:#04x}, keys={:?}",
                                        report.modifier,
                                        report.keys
                                    );
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendKeyboard(report)));
                                }
                                HidAction::KeyRelease => {
                                    let report = KeyboardReport::default();
                                    log::debug!("Actor sending KeyRelease (empty report)");
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendKeyboard(report)));
                                }
                                HidAction::MouseMove(dx, dy) => {
                                    let report = MouseReport {
                                        x: dx,
                                        y: dy,
                                        ..Default::default()
                                    };
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendMouse(report)));
                                    let stop_report = MouseReport {
                                        ..Default::default()
                                    };
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendMouse(stop_report)));
                                }
                                HidAction::MousePress(buttons) => {
                                    current_mouse_report.buttons = buttons;
                                    let _ = self.usb_hid_tx.send(AppEvent::UsbHidCommand(
                                        SendMouse(current_mouse_report),
                                    ));
                                }
                                HidAction::MouseRelease => {
                                    current_mouse_report = MouseReport::default();
                                    let _ = self.usb_hid_tx.send(AppEvent::UsbHidCommand(
                                        SendMouse(current_mouse_report),
                                    ));
                                }
                                HidAction::MouseWheel(amount) => {
                                    let report = MouseReport {
                                        wheel: amount,
                                        ..Default::default()
                                    };
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendMouse(report)));
                                    let stop_report = MouseReport {
                                        ..Default::default()
                                    };
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendMouse(stop_report)));
                                }
                                HidAction::ConsumerPress(usage_id) => {
                                    let report = ConsumerReport { usage: usage_id };
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendConsumer(report)));
                                }
                                HidAction::ConsumerRelease => {
                                    let report = ConsumerReport { usage: 0 };
                                    let _ = self
                                        .usb_hid_tx
                                        .send(AppEvent::UsbHidCommand(SendConsumer(report)));
                                }
                                HidAction::Delay(duration) => {
                                    thread::sleep(duration);
                                }
                            }
                        }
                    } else if let AppEvent::MappingUpdated(mapping_config) = app_event {
                        self.mapper.update_mapping_config(mapping_config);
                    } else {
                        log::warn!("Actor received unexpected event: {:?}", app_event);
                    }
                }
                Err(e) => {
                    log::error!("Actor failed to receive event: {}. Exiting.", e);
                    break;
                }
            }
        }
    }
}
