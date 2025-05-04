# Plan: User-Friendly Configuration via LittleFS and WebUSB

This plan outlines the steps to replace the hardcoded configurations (key mappings, Wi-Fi credentials, etc.) with a unified configuration loaded from the ESP32's internal flash using LittleFS, and allow updating this configuration from a PC companion web application via WebUSB.

**Chosen Technologies:**
*   **Storage:** LittleFS on internal ESP32 flash.
*   **Configuration Format:** JSON (Human-readable, widely supported by web technologies).
*   **Communication:** WebUSB (using TinyUSB stack).
*   **Serialization:** `serde` and `serde_json` crates.
*   **Rust Framework:** `esp-idf-svc` (providing safe wrappers).

**Configuration File:** `/littlefs/device_config.json`

---

## Phase 0: Prerequisites & Build System Configuration

1.  **Enable LittleFS Component:**
    *   Run `idf.py menuconfig` (ensure ESP-IDF env is sourced: `. ./export.sh`).
    *   Go to `Component config` -> `LittleFS Support`.
    *   Enable (`[*]`) this component. (If LittleFS is unavailable, enable `SPIFFS` instead and adjust partition subtype/mount function later).
    *   Note the default `Wear leveling sector size` (usually fine).
2.  **Configure Partition Table:**
    *   In `menuconfig`, go to `Component config` -> `Partition Table`.
    *   Select `[*] Custom partition table CSV`.
    *   Ensure `Partition table CSV file` points to `partitions.csv` (or your chosen name) in the project root.
    *   Create/Edit `partitions.csv` to include standard partitions (nvs, otadata, phy_init, ota_0, ota_1) and add a line for LittleFS storage:
        ```csv
        # Name,   Type, SubType, Offset,  Size, Flags
        # ... (nvs, otadata, phy_init, ota_0, ota_1 entries) ...
        storage,  data, littlefs,,        1M, # Label "storage", type data, subtype littlefs, size (e.g., 1M)
        ```
3.  **Enable TinyUSB WebUSB:**
    *   In `menuconfig`, go to `Component config` -> `TinyUSB Stack`.
    *   Ensure `[*] USB Device Communication Class (CDC)` is enabled.
    *   Ensure `[*] WebUSB` support is enabled.
    *   Verify VID/PID.
4.  **Add Dependencies:**
    *   Add `serde`, `serde_json` to `Cargo.toml`...
    *   The `esp-idf-svc` crate should already be a dependency.
5.  **Remove `toml_cfg`:** Remove `#[toml_cfg::toml_config]` for Wi-Fi...

## Phase 1: Unified Configuration Structure & Loading (with `esp-idf-svc`)

1.  **Define Unified Struct:**
    *   In a suitable module (e.g., `src/config.rs`), define a top-level struct `DeviceConfiguration`.
    *   Include fields for `mappings: MappingConfiguration` (the HashMap from `mapper.rs`) and `settings: DeviceSettings`.
    *   Define `DeviceSettings` struct containing fields like `wifi: Option<WifiSettings>`.
    *   Define `WifiSettings` struct containing `ssid: String` and `password: String`.
    *   Add `#[derive(Serialize, Deserialize, Debug, Clone)]` to all these new structs.
2.  **Initialize LittleFS (Rust):**
    *   In `main.rs` or a dedicated `storage` module, add code to:
        *   Import necessary items from `esp_idf_svc::vfs` and `esp_idf_svc::hal::prelude::*`.
        *   Use `esp_idf_svc::vfs::EspVfs::new()` to get VFS capability.
        *   Use `vfs.register_littlefs("/littlefs", "storage", partition.offset(), partition.len(), max_files)` (or similar API - check `esp-idf-svc` docs for exact signature) to mount the partition labeled `storage` at the `/littlefs` base path. Obtain partition details (`EspPartition::find_first(...)`).
3.  **Implement Configuration Loading (Rust):**
    *   In `main.rs` (or a `ConfigManager`), implement a function `load_device_configuration()`.
    *   Use standard Rust file I/O: `std::fs::File::open("/littlefs/device_config.json")`.
    *   Use `std::io::Read` trait methods to read data.
    *   Deserialize using `serde_json::from_slice` or `serde_json::from_reader`.
4.  **Handle Missing/Invalid Config & Defaults (Rust):**
    *   If loading fails:
        *   Log warning/error.
        *   Create a *default* `DeviceConfiguration` instance:
            *   Use `Mapper::load_default_config()` for the `mappings` field.
            *   Use `None` or default/empty values for `settings.wifi`.
        *   Serialize default config using `serde_json::to_vec` or `to_string`.
        *   Write default config using `std::fs::File::create("/littlefs/device_config.json")` and `std::io::Write` traits.
5.  **Distribute Configuration:**
    *   In `main.rs`, after successfully loading or creating the default `DeviceConfiguration`:
        *   Pass `loaded_config.mappings` to `Mapper::new()`.
        *   Pass `loaded_config.settings.wifi` to `Wifi::init()` (modify `Wifi::init` signature).
6.  **Test:** Verify boot, mount success logs, config loading/default creation, etc.

## Phase 2: WebUSB Firmware Implementation (Receiving Unified Config - with `esp-idf-svc`)

1.  **WebUSB Descriptors:**
    *   Ensure the USB descriptor configuration (likely in `bsp/usb_hid.rs` or similar) correctly includes the necessary WebUSB descriptors (BOS capability descriptor, etc.) as required by TinyUSB. Define a specific landing page URL allowed to connect.
2.  **TinyUSB WebUSB Task/Callbacks (Rust):**
    *   Implement the necessary TinyUSB callbacks for WebUSB:
        *   `tud_webusb_rx_cb()`: Called when data is received from the host over WebUSB.
        *   `tud_webusb_connected_cb()`: Optional callback for when a WebUSB connection is established.
    *   Alternatively, create a dedicated thread/task that polls `tud_webusb_available()` and reads data using `tud_webusb_read()`.
    *   **Data Transfer:** Reading data (`tud_webusb_rx_cb`, `tud_webusb_read`) and sending responses (`tud_webusb_write`) might require using `unsafe` blocks with functions from `esp_idf_svc::sys` (re-exporting `esp-idf-sys`), as safe wrappers may not exist in `esp-idf-svc` for these specific functions. Check the latest `esp-idf-svc` documentation.
3.  **Data Reception Protocol:**
    *   Define a simple protocol. Suggestion: **Length-Prefixed JSON**.
        *   The PC app first sends the total length of the JSON string (e.g., as a 4-byte unsigned integer, little-endian).
        *   The PC app then sends the raw JSON bytes.
    *   Read 4-byte length, then JSON bytes (using potentially `unsafe` calls to `tud_webusb_read`).
4.  **Deserialize and Save Unified Config (Rust):**
    *   Once JSON data is received (as `Vec<u8>`):
        *   Deserialize using `serde_json::from_slice` (safe Rust).
        *   If successful:
            *   Save to file using `std::fs::File::create` and `std::io::Write` (safe Rust).
            *   Trigger module reloads/updates (channels, shared state - safe Rust).
            *   *Optional:* Send success back via WebUSB TX (potentially `unsafe` `tud_webusb_write`).
        *   If deserialization fails:
            *   Log error.
            *   *Optional:* Send failure back to host.

## Phase 3: Basic Companion Web App (Sending Unified Config)

1.  **HTML Structure:**
    *   Add input fields for Wi-Fi SSID and Password.
    *   Keep `<textarea>` for pasting the *full* `DeviceConfiguration` JSON.
    *   A button to "Send Configuration".
    *   A status display area.
2.  **JavaScript Logic (`app.js`):**
    *   Use the `navigator.usb.requestDevice()` API to prompt the user to select the ESP32 device (filtered by VID/PID).
    *   Implement connection logic (`device.open()`, `device.selectConfiguration()`, `device.claimInterface()`). Handle potential errors.
    *   When "Send Configuration" is clicked:
        *   Get the *full* JSON string representing `DeviceConfiguration` from the `<textarea>`.
        *   *Important:* Ensure this JSON includes both the `mappings`