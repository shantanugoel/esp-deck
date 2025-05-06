use crate::config::DeviceConfiguration;
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
    pub config: DeviceConfiguration,
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
    pub config: DeviceConfiguration,
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
