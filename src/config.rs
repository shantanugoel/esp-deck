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
    Text(String, Option<String>),
    Image(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WidgetItemConfig {
    pub title: String,
    pub kind: WidgetKindConfig,
    pub update_interval_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)] // Note: PartialEq removed as Option<WidgetItemConfig> makes it harder if not needed for direct comparison of DeviceConfig
pub struct DeviceConfig {
    pub settings: DeviceSettings,
    pub mappings: MappingConfiguration,
    #[serde(default, deserialize_with = "deserialize_usize_key_map")]
    pub button_names: Option<HashMap<usize, String>>,
    // This field will hold Option<WidgetItemConfig> during deserialization and in-memory representation if needed.
    // For serialization to disk (config.json), we will ensure only Some(WidgetItemConfig) are written.
    #[serde(
        default,
        deserialize_with = "deserialize_usize_optional_widget_item_map"
    )]
    pub widgets: Option<HashMap<usize, Option<WidgetItemConfig>>>,
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
        let mut default_widgets_map = HashMap::new();
        default_widgets_map.insert(
            0,
            WidgetItemConfig {
                title: "Widget 1".to_string(),
                kind: WidgetKindConfig::Text(
                    "https://status.shantanugoel.com/api/v1/endpoints/internet_act-status/statuses"
                        .to_string(),
                    Some("/results/0/duration".to_string()),
                ),
                update_interval_seconds: 5,
            },
        );
        default_widgets_map.insert(
            1,
            WidgetItemConfig {
                title: "Widget 2".to_string(),
                kind: WidgetKindConfig::Image(
                    "https://www.gstatic.com/webp/gallery/2.jpg".to_string(),
                ),
                update_interval_seconds: 60,
            },
        );
        // Convert to HashMap<usize, Option<WidgetItemConfig>> for DeviceConfig.widgets
        let default_widgets_with_options: HashMap<usize, Option<WidgetItemConfig>> =
            default_widgets_map
                .into_iter()
                .map(|(k, v)| (k, Some(v)))
                .collect();

        let default_config = DeviceConfig {
            settings: DeviceSettings::default(),
            mappings: crate::mapper::Mapper::load_default_config(),
            button_names: Some(button_names),
            widgets: Some(default_widgets_with_options),
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
        config_to_save_from_request: &DeviceConfig, // This has widgets: Option<HashMap<usize, Option<WidgetItemConfig>>>
        config_updated_for: &mut ConfigUpdatedFor,
    ) -> Result<()> {
        log::info!("Updating configuration");

        let mut current_config_guard = self.config_data.lock().map_err(|e| {
            log::error!("Failed to lock config_data: {}", e);
            anyhow::anyhow!("Failed to lock config_data: {}", e)
        })?;

        // Clone current config to a mutable version that will be modified by merge_configs
        let mut merged_config_state = current_config_guard.clone();

        // Perform the merge. merged_config_state.widgets will be Option<HashMap<usize, Option<WidgetItemConfig>>>
        Self::merge_configs(
            &mut merged_config_state,
            config_to_save_from_request,
            config_updated_for,
        );

        // Prepare a version for serialization: filter out None widget items.
        // The struct to be serialized should have widgets: Option<HashMap<usize, WidgetItemConfig>>
        #[derive(Serialize)]
        struct DeviceConfigToSerialize<'a> {
            settings: &'a DeviceSettings,
            mappings: &'a MappingConfiguration,
            button_names: &'a Option<HashMap<usize, String>>,
            widgets: Option<HashMap<usize, WidgetItemConfig>>,
        }

        let widgets_to_serialize: Option<HashMap<usize, WidgetItemConfig>> = merged_config_state
            .widgets
            .as_ref()
            .map(|widgets_map_with_options| {
                widgets_map_with_options
                    .iter()
                    .filter_map(|(k, opt_v)| opt_v.as_ref().map(|v| (*k, v.clone())))
                    .collect::<HashMap<usize, WidgetItemConfig>>()
            })
            // Ensure an empty map is Some(HashMap::new()) rather than None if there were entries that all became None.
            // However, if the original map was None, or became empty after filtering, it should stay None or Some(empty).
            // The .collect() above will produce an empty HashMap if all items are None.
            // We need to handle if this collected map is empty, should it be None or Some(empty_map)?
            // If it's empty, let's make it None for cleaner JSON, unless an empty map `widgets: {}` is preferred over `widgets: null`.
            // The current logic in merge_configs (if target_map.values().all(Option::is_none)) already sets merged_config_state.widgets to None.
            // So, if merged_config_state.widgets is Some(empty_map_of_options), this map will result in Some(empty_map_of_items).
            // If merged_config_state.widgets is None, this will be None.
            .and_then(|map| if map.is_empty() { None } else { Some(map) });

        let config_for_serialization = DeviceConfigToSerialize {
            settings: &merged_config_state.settings,
            mappings: &merged_config_state.mappings,
            button_names: &merged_config_state.button_names,
            widgets: widgets_to_serialize,
        };

        let json_data = serde_json::to_vec_pretty(&config_for_serialization)?;

        // Backup existing config
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

        // Write the new config
        match std::fs::File::create(&self.config_path) {
            Ok(mut file) => {
                file.write_all(&json_data)?;
                log::info!("Configuration updated successfully.");
                // Update the in-memory state AFTER successful save
                *current_config_guard = merged_config_state;
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to save config file: {}", e);
                Err(anyhow::anyhow!("Failed to save config file: {}", e))
            }
        }
    }

    fn merge_configs(
        old_config: &mut DeviceConfig, // old_config.widgets is Option<HashMap<usize, Option<WidgetItemConfig>>>
        new_config: &DeviceConfig, // new_config.widgets is Option<HashMap<usize, Option<WidgetItemConfig>>>
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
                log::warn!("Key {} not found in old config for mappings, skipping", key);
            }
            config_updated_for.mappings = true;
        }

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

        // Handle widget updates: add, update, or delete (if value is None)
        if let Some(incoming_widgets_map) = &new_config.widgets {
            // incoming_widgets_map is &HashMap<usize, Option<WidgetItemConfig>>
            let target_map = old_config.widgets.get_or_insert_with(HashMap::new); // target_map is &mut HashMap<usize, Option<WidgetItemConfig>>

            for (key, optional_widget_item_config) in incoming_widgets_map {
                if let Some(widget_item_config) = optional_widget_item_config {
                    // Add or update: insert Some(widget_item_config)
                    target_map.insert(*key, Some(widget_item_config.clone()));
                } else {
                    // Value is None from incoming, signifies deletion. We remove the key.
                    // Or, if we want to represent deletion with Some(None) in target_map temporarily before save,
                    // we could do target_map.insert(*key, None); but removing is cleaner if the final saved state doesn't have nulls.
                    // Let's stick to inserting None to explicitly mark it, and filter on save.
                    target_map.insert(*key, None);
                }
            }
            // If after operations the map contains only None values or is empty, we might set old_config.widgets to None.
            // For now, let it be Some(map_with_potential_nones). Filtering happens at serialization for save.
            if target_map.is_empty() || target_map.values().all(Option::is_none) {
                old_config.widgets = None; // Or Some(empty_map) if we prefer that over None for the outer Option
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
        let config = self.config_data.lock().map_err(|e| {
            log::error!("Failed to lock config_data: {}", e);
            anyhow::anyhow!("Failed to lock config_data: {}", e)
        })?;
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
        let config_guard = self.config_data.lock().ok()?;
        config_guard
            .widgets
            .as_ref()
            .and_then(|widgets_map_with_options| {
                let cleaned_map: HashMap<usize, WidgetItemConfig> = widgets_map_with_options
                    .iter()
                    .filter_map(|(k, opt_v)| opt_v.as_ref().map(|v| (*k, v.clone())))
                    .collect();
                if cleaned_map.is_empty() {
                    None
                } else {
                    Some(cleaned_map)
                }
            })
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

// Custom deserializer for HashMap<usize, Option<WidgetItemConfig>>
fn deserialize_usize_optional_widget_item_map<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<usize, Option<WidgetItemConfig>>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into HashMap<String, Option<WidgetItemConfig>> first
    let map_str_keys: Option<HashMap<String, Option<WidgetItemConfig>>> =
        Option::deserialize(deserializer)?;

    match map_str_keys {
        Some(m) => {
            if m.is_empty() {
                Ok(None) // Keep it as None if the map is empty after deserialization
            } else {
                let converted_map: Result<HashMap<usize, Option<WidgetItemConfig>>, _> = m
                    .into_iter()
                    .map(|(k, v)| {
                        k.parse::<usize>()
                            .map_err(|_| de::Error::custom(format!("invalid usize key: {}", k)))
                            .map(|num_key| (num_key, v))
                    })
                    .collect();
                converted_map.map(Some)
            }
        }
        None => Ok(None), // If the whole 'widgets' field was null or not present
    }
}
