use std::sync::mpsc::{Receiver, Sender, SyncSender};

use crate::bsp::usb::{send_usb_message, UsbMessageError};
use crate::config::{ConfigUpdatedFor, Configurator, DeviceConfig, WifiSettings};
use crate::events::AppEvent;
use serde::{Deserialize, Serialize};

//Major version: 1, Minor version: 0
const PROTOCOL_VERSION: u32 = 0x00010000;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ProtocolHeader {
    pub version: u32,
    #[serde(rename = "correlationId", skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<u64>,
}

// Commands

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Command {
    GetConfig(GetConfigCommand),
    SetConfig(SetConfigCommand),
    ResetConfig(ResetConfigCommand),
    Reboot(RebootCommand),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetConfigCommand {
    pub header: ProtocolHeader,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetConfigCommand {
    pub header: ProtocolHeader,
    pub config: DeviceConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResetConfigCommand {
    pub header: ProtocolHeader,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RebootCommand {
    pub header: ProtocolHeader,
}

// Responses

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Response {
    Config(GetConfigResponse),
    Error(ErrorResponse),
    Ack(AckResponse),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetConfigResponse {
    pub header: ProtocolHeader,
    pub config: DeviceConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ErrorResponse {
    pub header: ProtocolHeader,
    pub message: String,
    #[serde(rename = "errorCode")]
    pub error_code: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AckResponse {
    pub header: ProtocolHeader,
    pub message: String,
    pub success: bool,
}

pub struct ProtocolManager<'a> {
    message_rx: Receiver<Vec<u8>>,
    main_wifi_time_init_tx: SyncSender<Option<WifiSettings>>,
    actor_tx: Sender<AppEvent>,
    config: &'a Configurator,
}

impl<'a> ProtocolManager<'a> {
    pub fn new(
        message_rx: Receiver<Vec<u8>>,
        main_wifi_time_init_tx: SyncSender<Option<WifiSettings>>,
        actor_tx: Sender<AppEvent>,
        config: &'a Configurator,
    ) -> Self {
        Self {
            message_rx,
            main_wifi_time_init_tx,
            actor_tx,
            config,
        }
    }

    pub fn run(&self) {
        loop {
            let message = self.message_rx.recv().unwrap();
            match serde_json::from_slice::<Command>(&message) {
                Ok(command) => {
                    log::info!("Received command: {:?}", command);
                    self.process_command(&command);
                }
                Err(e) => {
                    log::error!("Error deserializing command: {}", e);
                }
            }
        }
    }

    fn process_command(&self, command: &Command) {
        match command {
            Command::GetConfig(command) => {
                let response = GetConfigResponse {
                    header: ProtocolHeader {
                        version: PROTOCOL_VERSION,
                        correlation_id: command.header.correlation_id,
                    },

                    // TODO: Check error here
                    config: self.config.get_config().unwrap(),
                };
                let response_message = serde_json::to_vec(&response).unwrap();
                send_response(response_message);
            }

            Command::SetConfig(command) => {
                let response_header = ProtocolHeader {
                    version: PROTOCOL_VERSION,
                    correlation_id: command.header.correlation_id,
                };
                let new_config = command.config.clone();
                let mut config_updated_for = ConfigUpdatedFor::default();
                let response = match self.config.save(&new_config, &mut config_updated_for) {
                    Ok(_) => {
                        if self
                            .actor_tx
                            .send(AppEvent::MappingUpdated(Box::new(new_config.mappings)))
                            .is_err()
                        {
                            log::error!("Error sending mapping updated event. Will need to reboot for updated mappings to take effect");
                        }
                        let response = AckResponse {
                            header: response_header,
                            message: "Config set successfully".to_string(),
                            success: true,
                        };
                        serde_json::to_vec(&response).unwrap()
                    }
                    Err(e) => {
                        log::error!("Error saving config: {}", e);
                        let response = ErrorResponse {
                            header: response_header,
                            message: e.to_string(),
                            error_code: 1,
                        };
                        serde_json::to_vec(&response).unwrap()
                    }
                };
                send_response(response);
                if config_updated_for.wifi {
                    self.main_wifi_time_init_tx
                        .send(self.config.get_wifi_settings())
                        .unwrap();
                }
            }

            Command::ResetConfig(command) => {
                let response_header = ProtocolHeader {
                    version: PROTOCOL_VERSION,
                    correlation_id: command.header.correlation_id,
                };
                let mut response = AckResponse {
                    header: response_header,
                    ..Default::default()
                };
                match self.config.reset_config() {
                    Ok(_) => {
                        response.message = "Config reset successfully".to_string();
                        response.success = true;
                    }
                    Err(e) => {
                        log::error!("Error resetting config: {}", e);
                        response.message = e.to_string();
                        response.success = false;
                    }
                }
                let response_message = serde_json::to_vec(&response).unwrap();
                send_response(response_message);
            }

            Command::Reboot(command) => {
                let response_header = ProtocolHeader {
                    version: PROTOCOL_VERSION,
                    correlation_id: command.header.correlation_id,
                };
                let mut response = AckResponse {
                    header: response_header,
                    ..Default::default()
                };
                response.message = "Device will reboot".to_string();
                response.success = true;
                let response_message = serde_json::to_vec(&response).unwrap();
                send_response(response_message);
                esp_restart();
            }
        }
    }
}

fn send_response(response_message: Vec<u8>) {
    match send_usb_message(response_message) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error sending response: {}", e);
            match e.downcast_ref::<UsbMessageError>() {
                Some(UsbMessageError::NotEnoughSpace) => {
                    log::error!("Not enough space to send response");
                    //TODO This code won't be reached because of the FIFO not implemented in esp-tinyusb
                    // If it is implemented, we should try again later.
                }
                _ => {
                    log::error!("Discarding response");
                }
            }
        }
    }
}

#[cfg(target_arch = "xtensa")]
fn esp_restart() {
    unsafe { esp_idf_svc::sys::esp_restart() };
}
