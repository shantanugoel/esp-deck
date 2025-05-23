use crate::events::HidAction;
use keycode::{KeyMap, KeyMappingCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration; // Add serde for future file loading // Make sure this path is correct

// --- Configuration Structures ---
// These structs define how the mapping is represented, potentially loaded from a file later.

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ConfigAction {
    KeyPress {
        keys: Vec<String>,
        modifier: Option<String>,
    }, // Use string names for keys/modifiers
    KeyRelease,
    MouseMove {
        dx: i8,
        dy: i8,
    },
    MousePress {
        button: u8,
    }, // Button bitmask (1=Left, 2=Right, 4=Middle)
    MouseRelease,
    MouseWheel {
        amount: i8,
    },
    ConsumerPress {
        usage_id: u16,
    },
    ConsumerRelease,
    Delay {
        ms: u64,
    },
    SendString {
        keys: Vec<String>,
        modifiers: Vec<String>,
    },
    Sequence(Vec<ConfigAction>), // Represents a macro
}

// Define the type alias publicly here
pub type MappingConfiguration = HashMap<String, Vec<ConfigAction>>;

// --- Mapper Implementation ---

pub struct Mapper {
    config: MappingConfiguration,
    // TODO: Add state if needed for complex macros/toggles later
}

impl Mapper {
    /// Creates a new Mapper instance with the provided configuration.
    pub fn new(config: MappingConfiguration) -> Self {
        // Remove internal call to load_default_config
        // let default_config = Self::load_default_config();
        Self { config }
    }

    pub fn update_mapping_config(&mut self, config: MappingConfiguration) {
        self.config = config;
    }

    pub fn get_default_button_names() -> Vec<&'static str> {
        vec![
            "A",            // Button 1: Key A
            "Ctrl+C",       // Button 2: Copy
            "Hello",        // Button 3: Hello
            "F5",           // Button 4: F5
            "Left Click",   // Button 5: Mouse left
            "Move Square",  // Button 6: Mouse move
            "Scroll Down",  // Button 7: Scroll
            "Right Click",  // Button 8: Mouse right
            "Vol Up",       // Button 9: Volume up
            "Vol Down",     // Button 10: Volume down
            "Mute",         // Button 11: Mute
            "Play/Pause",   // Button 12: Play/Pause
            "Alt+Tab",      // Button 13: Alt+Tab
            "Ctrl+Alt+Del", // Button 14: Ctrl+Alt+Del
            "Copy+Paste",   // Button 15: Copy+Paste
            "TaskMgr",      // Button 16: Task Manager
        ]
    }

    /// Loads a hardcoded default configuration - Keep as helper, maybe make pub?
    /// Or move this logic entirely into config.rs's DeviceConfiguration::default_config()
    pub fn load_default_config() -> MappingConfiguration {
        let mut config = HashMap::new();

        // === Keyboard Examples ===

        // Button 1: Simple Key 'A' (Shift + KeyA)
        config.insert(
            "1".to_string(),
            vec![
                ConfigAction::KeyPress {
                    keys: vec!["KeyA".to_string()],
                    modifier: Some("ShiftLeft".to_string()),
                },
                ConfigAction::Delay { ms: 10 },
                ConfigAction::KeyRelease,
            ],
        );

        // Button 2: Key with Modifier (Ctrl+C)
        config.insert(
            "2".to_string(),
            vec![
                ConfigAction::KeyPress {
                    keys: vec!["KeyC".to_string()],
                    modifier: Some("ControlLeft".to_string()),
                },
                ConfigAction::Delay { ms: 10 },
                ConfigAction::KeyRelease,
            ],
        );

        // Button 3: Type a short string "Hello"
        config.insert(
            "3".to_string(),
            vec![ConfigAction::SendString {
                keys: vec![
                    "KeyH".to_string(),
                    "KeyE".to_string(),
                    "KeyL".to_string(),
                    "KeyL".to_string(),
                    "KeyO".to_string(),
                ],
                modifiers: vec![
                    "ShiftLeft".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                ],
            }],
        );

        // Button 4: Function Key F5
        config.insert(
            "4".to_string(),
            vec![
                ConfigAction::KeyPress {
                    keys: vec!["F5".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 10 },
                ConfigAction::KeyRelease,
            ],
        );

        // === Mouse Examples ===

        // Button 5: Left Mouse Click
        config.insert(
            "5".to_string(),
            vec![
                ConfigAction::MousePress { button: 1 }, // 1 = Left button
                ConfigAction::Delay { ms: 10 },
                ConfigAction::MouseRelease,
            ],
        );

        // Button 6: Mouse square movement (Original Button 2)
        config.insert(
            "6".to_string(),
            vec![
                ConfigAction::MouseMove { dx: 0, dy: 15 },
                ConfigAction::Delay { ms: 100 },
                ConfigAction::MouseMove { dx: 15, dy: 0 },
                ConfigAction::Delay { ms: 100 },
                ConfigAction::MouseMove { dx: 0, dy: -15 },
                ConfigAction::Delay { ms: 100 },
                ConfigAction::MouseMove { dx: -15, dy: 0 },
                ConfigAction::Delay { ms: 100 },
            ],
        );

        // Button 7: Scroll Wheel Down
        config.insert(
            "7".to_string(),
            vec![
                ConfigAction::MouseWheel { amount: -5 }, // Negative for down
                ConfigAction::Delay { ms: 10 },
                // Release/stop is handled by Actor sending 0 wheel movement
            ],
        );

        // Button 8: Right Mouse Click
        config.insert(
            "8".to_string(),
            vec![
                ConfigAction::MousePress { button: 2 }, // 2 = Right button
                ConfigAction::Delay { ms: 10 },
                ConfigAction::MouseRelease,
            ],
        );

        // === Consumer Control Examples ===

        // Button 9: Volume Up (Original Default)
        config.insert(
            "9".to_string(), // Use "default" for unassigned buttons
            vec![
                ConfigAction::ConsumerPress { usage_id: 0xE9 }, // Volume Increment
                ConfigAction::Delay { ms: 10 },
                ConfigAction::ConsumerRelease,
            ],
        );

        // Button 10: Volume Down
        config.insert(
            "10".to_string(),
            vec![
                ConfigAction::ConsumerPress { usage_id: 0xEA }, // Volume Decrement
                ConfigAction::Delay { ms: 10 },
                ConfigAction::ConsumerRelease,
            ],
        );

        // Button 11: Mute
        config.insert(
            "11".to_string(),
            vec![
                ConfigAction::ConsumerPress { usage_id: 0xE2 }, // Mute
                ConfigAction::Delay { ms: 10 },
                ConfigAction::ConsumerRelease,
            ],
        );

        // Button 12: Play/Pause
        config.insert(
            "12".to_string(),
            vec![
                ConfigAction::ConsumerPress { usage_id: 0xCD }, // Play/Pause
                ConfigAction::Delay { ms: 10 },
                ConfigAction::ConsumerRelease,
            ],
        );

        // === Sequence/Macro Examples ===

        // Button 13: Alt+Tab (Task Switch)
        config.insert(
            "13".to_string(),
            vec![
                ConfigAction::KeyPress {
                    keys: vec!["Tab".to_string()],
                    modifier: Some("AltLeft".to_string()),
                },
                ConfigAction::Delay { ms: 10 },
                ConfigAction::KeyRelease, // Release both Alt and Tab
            ],
        );

        // Button 14: Ctrl+Alt+Delete (Example - Careful!)
        config.insert(
            "14".to_string(),
            vec![
                // Press Ctrl
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Press Alt (Ctrl still held)
                ConfigAction::KeyPress {
                    keys: vec!["AltLeft".to_string()],
                    modifier: Some("ControlLeft".to_string()),
                },
                ConfigAction::Delay { ms: 5 },
                // Press Delete (Ctrl+Alt still held)
                ConfigAction::KeyPress {
                    keys: vec!["Delete".to_string()],
                    modifier: Some("ControlLeft AltLeft".to_string()),
                }, // Note: Multi-modifier needs testing
                ConfigAction::Delay { ms: 10 },
                // Release Delete (Ctrl+Alt remain)
                ConfigAction::KeyPress {
                    keys: vec!["AltLeft".to_string()],
                    modifier: Some("ControlLeft".to_string()),
                },
                ConfigAction::Delay { ms: 5 },
                // Release Alt (Ctrl remains)
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Release Ctrl
                ConfigAction::KeyRelease,
            ],
        );

        // Button 15: Copy, wait, Paste (Ctrl+C, wait, Ctrl+V)
        config.insert(
            "15".to_string(),
            vec![
                // --- Copy ---
                // Press Ctrl
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Press C (Ctrl Held)
                ConfigAction::KeyPress {
                    keys: vec!["KeyC".to_string()],
                    modifier: Some("ControlLeft".to_string()),
                },
                ConfigAction::Delay { ms: 10 },
                // Release C (Ctrl Held)
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Release Ctrl
                ConfigAction::KeyRelease,
                // --- Wait ---
                ConfigAction::Delay { ms: 100 },
                // --- Paste ---
                // Press Ctrl
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Press V (Ctrl Held)
                ConfigAction::KeyPress {
                    keys: vec!["KeyV".to_string()],
                    modifier: Some("ControlLeft".to_string()),
                },
                ConfigAction::Delay { ms: 10 },
                // Release V (Ctrl Held)
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Release Ctrl
                ConfigAction::KeyRelease,
            ],
        );

        // Button 16: Open Task Manager (Ctrl+Shift+Esc)
        config.insert(
            "16".to_string(),
            vec![
                // Press Ctrl
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Press Shift (Ctrl Held)
                ConfigAction::KeyPress {
                    keys: vec!["ShiftLeft".to_string()],
                    modifier: Some("ControlLeft".to_string()),
                },
                ConfigAction::Delay { ms: 5 },
                // Press Esc (Ctrl+Shift Held)
                ConfigAction::KeyPress {
                    keys: vec!["Escape".to_string()],
                    modifier: Some("ControlLeft ShiftLeft".to_string()),
                }, // Note: Multi-modifier needs testing
                ConfigAction::Delay { ms: 10 },
                // Release Esc (Ctrl+Shift Held)
                ConfigAction::KeyPress {
                    keys: vec!["ShiftLeft".to_string()],
                    modifier: Some("ControlLeft".to_string()),
                },
                ConfigAction::Delay { ms: 5 },
                // Release Shift (Ctrl Held)
                ConfigAction::KeyPress {
                    keys: vec!["ControlLeft".to_string()],
                    modifier: None,
                },
                ConfigAction::Delay { ms: 5 },
                // Release Ctrl
                ConfigAction::KeyRelease,
            ],
        );

        // Default for any unassigned buttons (e.g., > 16)
        config.insert(
            "default".to_string(),
            vec![
                // Do nothing, or maybe a visual cue if UI supports it?
                ConfigAction::Delay { ms: 1 }, // Minimal action
            ],
        );

        config
    }

    /// Translates a configuration key string (e.g., "a", "LCtrl") and optional modifier string.
    /// Returns (modifier_bitmask, key_code).
    /// key_code will be 0 if the key_name itself represents a modifier.
    fn translate_key(key_name: &str, mod_name: Option<&str>) -> (u8, u8) {
        log::debug!(
            "translate_key called: key_name=\"{}\", mod_name={:?}",
            key_name,
            mod_name
        );
        // 1. Parse key string to KeyMappingCode
        let key_code_enum = match KeyMappingCode::from_str(key_name) {
            Ok(kc) => kc,
            Err(_) => {
                log::warn!("Invalid key name string: {}", key_name);
                return (0, 0);
            }
        };
        // 2. Convert KeyMappingCode to KeyMap
        let key_map = KeyMap::from(key_code_enum);
        // 3. Get potential USB keycode
        let key_code = key_map.usb as u8;

        // 4. Determine modifier bitmask by parsing mod_name (potentially multiple, space-separated)
        let mut final_modifier_bitmask = match mod_name {
            Some(m_names) => {
                let mut combined_mask = 0u8;
                for m_name in m_names.split_whitespace() {
                    // Split by whitespace
                    match KeyMappingCode::from_str(m_name) {
                        Ok(mod_code_enum) => {
                            let mod_map = KeyMap::from(mod_code_enum);
                            match mod_map {
                                KeyMap {
                                    usb: 0xE0..=0xE7, ..
                                } => {
                                    combined_mask |= 1 << (mod_map.usb - 0xE0); // OR the bits together
                                }
                                _ => {
                                    log::warn!("Part '{}' in modifier string '{}' is not a standard modifier key.", m_name, m_names);
                                }
                            }
                        }
                        Err(_) => {
                            log::warn!(
                                "Invalid modifier name part '{}' in string '{}'",
                                m_name,
                                m_names
                            );
                        }
                    }
                }
                combined_mask // Return the combined mask
            }
            None => 0,
        };

        // 5. Check if the main key_name itself is a modifier
        if (0xE0..=0xE7).contains(&key_code) {
            // It IS a modifier. Add its bit to the mask.
            final_modifier_bitmask |= 1 << (key_code - 0xE0);
            // Return the mask and 0 for the keycode (modifiers don't go in keys array)
            return (final_modifier_bitmask, 0);
        }

        // 6. If we got here, the key_name was NOT a modifier.
        log::debug!(
            "translate_key returning: modifier={:#04x}, keycode={:#04x}",
            final_modifier_bitmask,
            key_code
        );
        (final_modifier_bitmask, key_code)
    }

    /// Retrieves the sequence of primitive HidActions for a given button ID.
    pub fn get_action_sequence(&self, button_id: i32) -> Vec<HidAction> {
        let key = button_id.to_string();
        let config_sequence = self
            .config
            .get(&key)
            .or_else(|| self.config.get("default")) // Fallback to default if specific ID not found
            .cloned() // Clone the sequence to avoid borrowing issues
            .unwrap_or_default(); // Return empty sequence if neither found

        Self::translate_sequence(config_sequence)
    }

    /// Recursively translates a sequence of ConfigActions into HidActions.
    fn translate_sequence(config_actions: Vec<ConfigAction>) -> Vec<HidAction> {
        let mut hid_actions = Vec::new();
        for action in config_actions {
            match action {
                ConfigAction::KeyPress { keys, modifier } => {
                    let mut keycodes = [0u8; 6];
                    let mut mod_bits = 0u8;
                    for (idx, key) in keys.iter().take(6).enumerate() {
                        let (mb, key_code) = Self::translate_key(key, modifier.as_deref());
                        keycodes[idx] = key_code;
                        mod_bits |= mb;
                    }
                    hid_actions.push(HidAction::KeyPress(mod_bits, keycodes));
                }
                ConfigAction::KeyRelease => hid_actions.push(HidAction::KeyRelease),
                ConfigAction::MouseMove { dx, dy } => {
                    hid_actions.push(HidAction::MouseMove(dx, dy))
                }
                ConfigAction::MousePress { button } => {
                    hid_actions.push(HidAction::MousePress(button))
                }
                ConfigAction::MouseRelease => hid_actions.push(HidAction::MouseRelease),
                ConfigAction::MouseWheel { amount } => {
                    hid_actions.push(HidAction::MouseWheel(amount))
                }
                ConfigAction::ConsumerPress { usage_id } => {
                    hid_actions.push(HidAction::ConsumerPress(usage_id))
                }
                ConfigAction::ConsumerRelease => hid_actions.push(HidAction::ConsumerRelease),
                ConfigAction::Delay { ms } => {
                    hid_actions.push(HidAction::Delay(Duration::from_millis(ms)))
                }
                ConfigAction::SendString { keys, modifiers } => {
                    for (key, modifier) in keys.iter().zip(modifiers.iter()) {
                        let modifier = if !modifier.is_empty() {
                            Some(modifier.as_str())
                        } else {
                            None
                        };
                        let (mb, key_code) = Self::translate_key(key, modifier);
                        hid_actions.push(HidAction::KeyPress(mb, [key_code, 0, 0, 0, 0, 0]));
                        hid_actions.push(HidAction::KeyRelease);
                    }
                }
                ConfigAction::Sequence(sub_sequence) => {
                    hid_actions.extend(Self::translate_sequence(sub_sequence));
                }
            }
        }
        hid_actions
    }
}
