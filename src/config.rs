use crate::mapper::MappingConfiguration;
use anyhow::Result;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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
    pub api_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WidgetKindConfig {
    Text(String),
    Image(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WidgetItemConfig {
    pub title: String,
    pub kind: WidgetKindConfig,
    pub update_interval_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DeviceConfig {
    pub settings: DeviceSettings,
    pub mappings: MappingConfiguration,
    // Custom deserializer because JS sends strings for the keys
    #[serde(default, deserialize_with = "deserialize_usize_key_map")]
    pub button_names: Option<HashMap<usize, String>>,
    pub widgets: Option<HashMap<usize, WidgetItemConfig>>,
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
    pub button_names: bool,
    pub api_key: bool,
    pub widgets: bool,
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
        let mut button_names = HashMap::new();
        for (i, name) in crate::mapper::Mapper::get_default_button_names()
            .iter()
            .enumerate()
        {
            button_names.insert(i, name.to_string());
        }
        // TODO: Remove this once we have a real default widget
        let mut default_widgets = HashMap::new();
        default_widgets.insert(
            0,
            WidgetItemConfig {
                title: "Widget 1".to_string(),
                kind: WidgetKindConfig::Text("bcd".to_string()),
                update_interval_seconds: 3600,
            },
        );
        default_widgets.insert(
            1,
            WidgetItemConfig {
                title: "Widget 2".to_string(),
                kind: WidgetKindConfig::Text("def".to_string()),
                update_interval_seconds: 3600,
            },
        );
        let default_config = DeviceConfig {
            settings: DeviceSettings::default(),
            mappings: crate::mapper::Mapper::load_default_config(),
            button_names: Some(button_names),
            widgets: Some(default_widgets),
        };
        log::info!("Creating default configuration file at {}", config_path);

        let dir_path = match std::path::Path::new(config_path).parent() {
            Some(p) => p,
            None => {
                log::error!("No parent directory for config_path: {}", config_path);
                return Err(anyhow::anyhow!("No parent directory for config_path"));
            }
        };
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
        let mut old_config = match self.config_data.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock config_data: {}", e);
                return Err(anyhow::anyhow!("Failed to lock config_data: {}", e));
            }
        };
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
            old_config.settings.timezone_offset = Some(*new_timezone_offset);
            config_updated_for.timezone_offset = true;
        }
        if let Some(new_api_key) = &new_config.settings.api_key {
            if new_api_key.is_empty() {
                old_config.settings.api_key = None;
            } else {
                old_config.settings.api_key = Some(new_api_key.clone());
            }
            config_updated_for.api_key = true;
        }
        for (key, new_actions) in &new_config.mappings {
            if old_config.mappings.contains_key(key) {
                old_config.mappings.insert(key.clone(), new_actions.clone());
            } else {
                log::warn!("Key {} not found in old config, skipping", key);
            }
            config_updated_for.mappings = true;
        }

        // Merge button_names as a map, only update provided keys
        if let Some(new_names) = &new_config.button_names {
            let old_names = old_config.button_names.get_or_insert_with(HashMap::new);
            for (&idx, name) in new_names {
                let truncated = if name.len() > 20 {
                    name[..20].to_string()
                } else {
                    name.clone()
                };
                old_names.insert(idx, truncated);
            }
            config_updated_for.button_names = true;
        }
        if let Some(new_widgets) = &new_config.widgets {
            let old_widgets = old_config.widgets.get_or_insert_with(HashMap::new);
            for (key, value) in new_widgets {
                old_widgets.insert(key.clone(), value.clone());
            }
            config_updated_for.widgets = true;
        }
    }

    pub fn reset_config(&self) -> Result<()> {
        let mut config = match self.config_data.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock config_data: {}", e);
                return Err(anyhow::anyhow!("Failed to lock config_data: {}", e));
            }
        };
        match Self::create_and_save_default_config(&self.config_path) {
            Ok(default_config) => {
                *config = default_config;
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to reset config: {}", e);
                Err(e)
            }
        }
    }

    pub fn get_config(&self) -> Result<DeviceConfig> {
        let config = match self.config_data.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("Failed to lock config_data: {}", e);
                return Err(anyhow::anyhow!("Failed to lock config_data: {}", e));
            }
        };
        Ok(config.clone())
    }

    pub fn get_wifi_settings(&self) -> Option<WifiSettings> {
        let config = self.config_data.lock().ok()?;
        config.settings.wifi.clone()
    }

    pub fn get_timezone_offset(&self) -> Option<f32> {
        let config = self.config_data.lock().ok()?;
        config.settings.timezone_offset
    }

    pub fn get_mappings(&self) -> Option<MappingConfiguration> {
        let config = self.config_data.lock().ok()?;
        Some(config.mappings.clone())
    }

    pub fn get_button_names(&self) -> Option<HashMap<usize, String>> {
        let config = self.config_data.lock().ok()?;
        config.button_names.clone()
    }

    pub fn get_api_key(&self) -> Option<String> {
        let config = self.config_data.lock().ok()?;
        config.settings.api_key.clone()
    }

    pub fn get_widgets(&self) -> Option<HashMap<usize, WidgetItemConfig>> {
        let config = self.config_data.lock().ok()?;
        config.widgets.clone()
    }
}

// Custom deserializer for HashMap<usize, String> from JSON with string keys
fn deserialize_usize_key_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<usize, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<String, String> = HashMap::deserialize(deserializer)?;
    if map.is_empty() {
        Ok(None)
    } else {
        let converted: Result<HashMap<usize, String>, D::Error> = map
            .into_iter()
            .map(|(k, v)| {
                k.parse::<usize>()
                    .map_err(|_| de::Error::custom(format!("invalid usize key: {}", k)))
                    .map(|num| (num, v))
            })
            .collect();
        converted.map(Some)
    }
}
