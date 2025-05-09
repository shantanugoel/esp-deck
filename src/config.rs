use crate::mapper::MappingConfiguration;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    sync::{Arc, Mutex},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WifiSettings {
    pub ssid: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)] // Default for easy creation
pub struct DeviceSettings {
    // Add optional settings here
    pub wifi: Option<WifiSettings>,
    pub timezone_offset: Option<f32>,
    // pub display_brightness: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeviceConfig {
    pub settings: DeviceSettings,
    pub mappings: MappingConfiguration,
}

#[derive(Debug, Clone)]
pub struct Configurator {
    // Config path shouldn't be needed outside of this module, so this
    // is a private field.
    config_path: String,
    config_data: Arc<Mutex<DeviceConfig>>,
}

// Helper function to create a default configuration object
impl Configurator {
    /// Loads the device configuration from LittleFS, or creates and saves a default config if not found/invalid.
    pub fn load_or_create_default_config(config_path: &str) -> Result<Self> {
        let config = match std::fs::File::open(config_path) {
            Ok(mut file) => {
                log::info!("Reading configuration from {}", config_path);
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                match serde_json::from_slice(&buf) {
                    Ok(config) => {
                        log::info!("Configuration loaded successfully.");
                        config
                    }
                    Err(e) => {
                        log::error!("Failed to parse config file: {}. Using default config.", e);
                        Self::create_and_save_default_config(config_path)?
                    }
                }
            }
            Err(e) => {
                log::warn!(
                    "Config file not found ({}): {}. Creating default config.",
                    config_path,
                    e
                );
                Self::create_and_save_default_config(config_path)?
            }
        };
        Ok(Configurator {
            config_path: config_path.to_string(),
            config_data: Arc::new(Mutex::new(config)),
        })
    }

    /// Creates a default configuration and saves it to the filesystem.
    fn create_and_save_default_config(config_path: &str) -> Result<DeviceConfig> {
        let default_config = DeviceConfig {
            settings: DeviceSettings::default(), // Default settings (e.g., no wifi)
            mappings: crate::mapper::Mapper::load_default_config(), // Default mappings
        };
        log::info!("Creating default configuration file at {}", config_path);

        let dir_path = std::path::Path::new(config_path).parent().unwrap();
        if let Err(e) = std::fs::create_dir_all(dir_path) {
            log::warn!(
                "Failed to create directory {}: {} (continuing anyway)",
                dir_path.display(),
                e
            );
        }

        match std::fs::File::create(config_path) {
            Ok(mut file) => {
                let json_data = serde_json::to_vec_pretty(&default_config)?;
                file.write_all(&json_data)?;
                log::info!("Default configuration saved successfully.");
            }
            Err(e) => {
                log::error!("Failed to save default config file: {}", e);
            }
        }
        Ok(default_config)
    }

    pub fn save(&self) -> Result<()> {
        log::info!("Updating configuration");

        // Backup the existing config file
        let backup_path = format!("{}.bak", self.config_path);
        std::fs::copy(&self.config_path, &backup_path)?;
        log::info!("Backup of existing config saved to {}", backup_path);

        // Save the new config
        let config = self.config_data.lock().unwrap();
        let json_data = serde_json::to_vec_pretty(&config.clone())?;
        match std::fs::File::open(&self.config_path) {
            Ok(mut file) => {
                file.write_all(&json_data)?;
                log::info!("Configuration updated successfully.");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to save config file: {}", e);
                Err(anyhow::anyhow!("Failed to save config file: {}", e))
            }
        }
    }

    pub fn get_config(&self) -> Result<DeviceConfig> {
        let config = self.config_data.lock().unwrap();
        Ok(config.clone())
    }

    pub fn get_wifi_settings(&self) -> Option<WifiSettings> {
        let config = self.config_data.lock().unwrap();
        config.settings.wifi.clone()
    }

    pub fn get_timezone_offset(&self) -> Option<f32> {
        let config = self.config_data.lock().unwrap();
        config.settings.timezone_offset.clone()
    }

    pub fn get_mappings(&self) -> Option<MappingConfiguration> {
        let config = self.config_data.lock().unwrap();
        Some(config.mappings.clone())
    }
}
