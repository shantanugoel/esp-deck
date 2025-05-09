use std::sync::mpsc::Receiver;

use crate::bsp::usb::{send_usb_message, UsbMessageError};
use crate::config::{Configurator, DeviceConfig};
use serde::{Deserialize, Serialize};

//Major version: 1, Minor version: 0
const PROTOCOL_VERSION: u32 = 0x00010000;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AckResponse {
    pub header: ProtocolHeader,
    pub message: String,
    pub success: bool,
}

pub struct ProtocolManager<'a> {
    message_rx: Receiver<Vec<u8>>,
    config: &'a Configurator,
}

impl<'a> ProtocolManager<'a> {
    pub fn new(message_rx: Receiver<Vec<u8>>, config: &'a Configurator) -> Self {
        Self { message_rx, config }
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
                let response = AckResponse {
                    header: ProtocolHeader {
                        version: PROTOCOL_VERSION,
                        correlation_id: command.header.correlation_id,
                    },
                    message: "Config set successfully".to_string(),
                    success: true,
                };
                let response_message = serde_json::to_vec(&response).unwrap();
                send_response(response_message);
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
