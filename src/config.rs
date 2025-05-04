use crate::mapper::MappingConfiguration;
use serde::{Deserialize, Serialize};

// --- Configuration Structures ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WifiSettings {
    pub ssid: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)] // Default for easy creation
pub struct DeviceSettings {
    // Add optional settings here
    pub wifi: Option<WifiSettings>,
    // pub timezone_offset: Option<f32>, // Example: Could move tz_offset here later
    // pub display_brightness: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceConfiguration {
    pub settings: DeviceSettings,
    pub mappings: MappingConfiguration,
}

// Helper function to create a default configuration object
impl DeviceConfiguration {
    pub fn default_config() -> Self {
        DeviceConfiguration {
            settings: DeviceSettings::default(), // Default settings (e.g., no wifi)
            mappings: crate::mapper::Mapper::load_default_config(), // Default mappings
        }
    }
}
