use crate::mapper::MappingConfiguration;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
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

#[derive(Debug, Clone, Default)]
pub struct ConfigUpdatedFor {
    pub wifi: bool,
    pub timezone_offset: bool,
    pub mappings: bool,
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

    pub fn save(
        &self,
        config: &DeviceConfig,
        config_updated_for: &mut ConfigUpdatedFor,
    ) -> Result<()> {
        log::info!("Updating configuration");

        // Save the new config
        let mut old_config = self.config_data.lock().unwrap();
        *old_config = config.clone();
        Self::merge_configs(&mut old_config, config, config_updated_for);
        let json_data = serde_json::to_vec_pretty(&old_config.clone())?;

        // Backup the existing config file
        let config_path = std::path::Path::new(&self.config_path);
        if config_path.exists() {
            log::info!("Backing up existing config file");
            let backup_path = format!("{}.bak", self.config_path);
            let mut src = fs::File::open(&self.config_path)?;
            let mut dst = fs::File::create(&backup_path)?;
            std::io::copy(&mut src, &mut dst)?;
            log::info!("Backup of existing config saved to {}", backup_path);
        } else {
            log::warn!("Config file not found, skipping backup");
        }

        match std::fs::File::create(&self.config_path) {
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

    fn merge_configs(
        old_config: &mut DeviceConfig,
        new_config: &DeviceConfig,
        config_updated_for: &mut ConfigUpdatedFor,
    ) {
        if let Some(new_wifi) = &new_config.settings.wifi {
            old_config.settings.wifi = Some(new_wifi.clone());
            config_updated_for.wifi = true;
        }
        if let Some(new_timezone_offset) = &new_config.settings.timezone_offset {
            old_config.settings.timezone_offset = Some(new_timezone_offset.clone());
            config_updated_for.timezone_offset = true;
        }
        for (key, new_actions) in &new_config.mappings {
            if old_config.mappings.contains_key(key) {
                old_config.mappings.insert(key.clone(), new_actions.clone());
            } else {
                log::warn!("Key {} not found in old config, skipping", key);
            }
        }
        config_updated_for.mappings = true;
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
