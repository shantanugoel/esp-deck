# Plan: User-Friendly Configuration via LittleFS and WebUSB

This plan outlines the steps to replace the hardcoded configurations (key mappings, Wi-Fi credentials, etc.) with a unified configuration loaded from the ESP32's internal flash using LittleFS, and allow updating this configuration from a PC companion web application via WebUSB.

**Chosen Technologies:**
*   **Storage:** LittleFS on internal ESP32 flash.
*   **Configuration Format:** JSON (Human-readable, widely supported by web technologies).
*   **Communication:** WebUSB (using TinyUSB stack).
*   **Serialization:** `serde` and `serde_json` crates.

**Configuration File:** `/littlefs/device_config.json`

---

## Phase 0: Prerequisites & Build System Configuration

1.  **Enable LittleFS:**
    *   Use `idf.py menuconfig` (or the VS Code extension equivalent).
    *   Go to `Component config` -> `Wear Levelling`.
    *   Enable `Enable Wear Levelling library`.
    *   Go to `Component config` -> `Partition Table`.
    *   Choose a partition table layout that includes a `storage` partition suitable for LittleFS (e.g., "Factory app, two OTA definitions + storage partition"). Note the label used (often `storage` or `spiffs`).
    *   Verify partition settings (size, offset).
2.  **Enable TinyUSB WebUSB:**
    *   Use `idf.py menuconfig`.
    *   Go to `Component config` -> `TinyUSB Stack`.
    *   Ensure `USB Device Communication Class (CDC)` is enabled (often needed as a base or for debugging).
    *   Ensure `WebUSB` support is enabled within the TinyUSB configuration options.
    *   Configure VID/PID if not already set correctly.
3.  **Add Dependencies:**
    *   Add `serde` and `serde_json` to `Cargo.toml` (with `derive` feature for `serde`).
    *   Ensure necessary `esp-idf-svc` features for VFS/LittleFS are potentially enabled if needed (might be included by default).
4.  **Remove `toml_cfg`:** Remove the `#[toml_cfg::toml_config]` macro and its usage for Wi-Fi credentials in `main.rs`, as these will now come from the loaded JSON config.

## Phase 1: Unified Configuration Structure & Loading

1.  **Define Unified Struct:**
    *   In a suitable module (e.g., `src/config.rs`), define a top-level struct `DeviceConfiguration`.
    *   Include fields for `mappings: MappingConfiguration` (the HashMap from `mapper.rs`) and `settings: DeviceSettings`.
    *   Define `DeviceSettings` struct containing fields like `wifi: Option<WifiSettings>`.
    *   Define `WifiSettings` struct containing `ssid: String` and `password: String`.
    *   Add `#[derive(Serialize, Deserialize, Debug, Clone)]` to all these new structs.
2.  **Initialize LittleFS:**
    *   In `main.rs` or a dedicated `storage` module, add code to:
        *   Initialize the Virtual Filesystem (VFS) layer for ESP-IDF.
        *   Configure and mount the LittleFS partition using the label defined in the partition table (e.g., `storage`). Use a base path like `/littlefs`.
3.  **Implement Configuration Loading:**
    *   In `main.rs` (or a `ConfigManager`), implement a function `load_device_configuration()`.
    *   Attempt to read `/littlefs/device_config.json`.
    *   Use `serde_json::from_slice/reader` to deserialize into the `DeviceConfiguration` struct.
4.  **Handle Missing/Invalid Config & Defaults:**
    *   If loading fails (file missing, invalid JSON):
        *   Log warning/error.
        *   Create a *default* `DeviceConfiguration` instance:
            *   Use `Mapper::load_default_config()` for the `mappings` field.
            *   Use `None` or default/empty values for `settings.wifi`.
        *   *Recommended:* Serialize this default `DeviceConfiguration` to JSON and write it to `/littlefs/device_config.json` so a valid file exists.
5.  **Distribute Configuration:**
    *   In `main.rs`, after successfully loading or creating the default `DeviceConfiguration`:
        *   Pass `loaded_config.mappings` to `Mapper::new()`.
        *   Pass `loaded_config.settings.wifi` to `Wifi::init()` (modify `Wifi::init` signature).
6.  **Test:** Verify boot, LittleFS mount, attempt to load config, creation/use of default config, and passing of defaults to `Mapper` and `Wifi` modules.

## Phase 2: WebUSB Firmware Implementation (Receiving Unified Config)

1.  **WebUSB Descriptors:**
    *   Ensure the USB descriptor configuration (likely in `bsp/usb_hid.rs` or similar) correctly includes the necessary WebUSB descriptors (BOS capability descriptor, etc.) as required by TinyUSB. Define a specific landing page URL allowed to connect.
2.  **TinyUSB WebUSB Task/Callbacks:**
    *   Implement the necessary TinyUSB callbacks for WebUSB:
        *   `tud_webusb_rx_cb()`: Called when data is received from the host over WebUSB.
        *   `tud_webusb_connected_cb()`: Optional callback for when a WebUSB connection is established.
    *   Alternatively, create a dedicated thread/task that polls `tud_webusb_available()` and reads data using `tud_webusb_read()`.
3.  **Data Reception Protocol:**
    *   Define a simple protocol. Suggestion: **Length-Prefixed JSON**.
        *   The PC app first sends the total length of the JSON string (e.g., as a 4-byte unsigned integer, little-endian).
        *   The PC app then sends the raw JSON bytes.
    *   In the ESP32 WebUSB receive callback/task:
        *   Read the 4-byte length prefix.
        *   Allocate a buffer of that size.
        *   Read exactly that many bytes into the buffer (handle potential partial reads).
4.  **Deserialize and Save Unified Config:**
    *   Once the full JSON data is received:
        *   Attempt to deserialize into `DeviceConfiguration` using `serde_json::from_slice`.
        *   If successful:
            *   Open `/littlefs/device_config.json` for writing.
            *   Write the *received* (and validated) JSON bytes to the file.
            *   *Crucial:* Trigger relevant modules to reload/apply the new settings (e.g., signal the Wi-Fi task to reconnect if credentials changed, potentially signal `Mapper` to reload, although less critical as it's usually read once at start). This might involve channels or shared state (`Arc<Mutex<...>>`).
            *   *Optional:* Send success back to host.
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
        *   *Important:* Ensure this JSON includes both the `mappings` and `settings` keys with their respective structures.
        *   Calculate length, create buffers, send length, send JSON data via `transferOut`...
    *   *Optional:* Implement logic to listen for success/failure messages back from the device using `device.transferIn()`.

## Phase 4: Full Companion Web App UI (Unified Config)

1.  **Mapping Editor UI:**
    *   Design and implement a user-friendly interface within the web app for creating and editing key mappings (buttons, dropdowns for actions/keys, input fields for delays/mouse movements, etc.). Avoid requiring users to write raw JSON.
2.  **Settings UI:**
    *   Add sections/forms in the web app UI for editing Wi-Fi credentials and any other future settings.
3.  **Configuration Generation:**
    *   Update JavaScript logic to generate a single JSON object conforming to the `DeviceConfiguration` structure, combining data from the mapping editor *and* the settings forms.
4.  **Integration:** Replace `<textarea>` with the combined UI. "Save to Device" generates the unified JSON and sends it.

## Phase 5: Refinements & Error Handling

1.  **Robustness:**
    *   Add specific error handling for filesystem writes, JSON validation failures, and failures applying settings (e.g., Wi-Fi connection failed with new credentials).
2.  **Feedback:**
    *   Provide feedback for settings application (e.g., "Wi-Fi connecting...", "Wi-Fi connected", "Wi-Fi failed").
3.  **Optimization:**
    *   Consider using MessagePack instead of JSON for smaller configuration size and faster parsing, if needed. Update serialization/deserialization on both ends.
4.  **Device Discovery:**
    *   Improve the WebUSB connection flow if multiple valid devices might be present.
5.  **Applying Settings:** Define a clear strategy for how firmware modules react to configuration changes *after* the new config is saved (re-init, reload flags, message passing, etc.).

--- 