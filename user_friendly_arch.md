# Architecture: User-Friendly Configuration via LittleFS and WebUSB

This document describes the architecture for loading a unified device configuration (mappings, settings) from LittleFS and updating it via a WebUSB-enabled companion web application.

## Components

*   **ESP32 Firmware:** The Rust application running on the ESP-Deck device.
    *   **`main` Module:** Initializes hardware, `esp-idf-svc` services (VFS), mounts LittleFS (via `esp-idf-svc::vfs`), loads the unified `DeviceConfiguration` (using `std::fs` + `serde_json`), distributes parts of the config (mappings to `Mapper`, Wi-Fi settings to `Wifi`), and spawns main application threads.
    *   **`Mapper` Module:** Initialized with `MappingConfiguration` data from the loaded `DeviceConfiguration`. Responsible for providing action sequences to the `Actor`.
    *   **`Wifi` Module:** Initialized with `WifiSettings` (SSID/password) from the loaded `DeviceConfiguration`. Manages Wi-Fi connection.
    *   **`Actor` Module:** Receives button press events from the UI, requests the corresponding action sequence from the `Mapper`, and executes the sequence by sending `HidAction` commands to the `UsbHidClient`.
    *   **`UsbHidClient` Module:** Sends HID reports (Keyboard, Mouse, Consumer) over USB based on commands received from the `Actor`.
    *   **`WebUSB Handler` (Component/Task):** Handles WebUSB communication.
        *   Listens for incoming data (likely via TinyUSB callbacks/polling using `esp_idf_svc::sys`).
        *   Implements length-prefix protocol (using `esp_idf_svc::sys` functions for reads).
        *   Uses `serde_json` to validate/deserialize the received data into `DeviceConfiguration`.
        *   If valid, writes JSON to `/littlefs/device_config.json` (using `std::fs`).
        *   **Crucially:** Needs a mechanism to notify other modules (e.g., `Wifi`) that settings *may* have changed, potentially requiring re-initialization or reconnection.
        *   *Optional:* Sends status back (using `esp_idf_svc::sys` functions for writes).
    *   **`LittleFS / VFS`:** Managed via `esp-idf-svc::vfs` APIs wrapping the underlying ESP-IDF components.
    *   **`TinyUSB Stack`:** Provides USB interfaces, configured via `menuconfig`, potentially interacted with directly via `esp_idf_svc::sys` for WebUSB data transfer.
    *   **`config` Module (New/Conceptual):** Defines the `DeviceConfiguration`, `DeviceSettings`, `WifiSettings` structs with `serde` derives.
*   **Companion Web Application:** A static web page (HTML, CSS, JavaScript) hosted on a PC.
    *   **UI:** Provides a user-friendly interface for editing *both* button mappings *and* device settings (like Wi-Fi).
    *   **Mapping & Settings Logic:** Translates UI state into a single JSON object conforming to the `DeviceConfiguration` structure.
    *   **WebUSB Communication:** Uses `navigator.usb` API to connect to the ESP32 device and transfer the length-prefixed `DeviceConfiguration` JSON data over the WebUSB OUT endpoint.
    *   *Optional:* Receives status updates from the device via the WebUSB IN endpoint.

## Data Flow Diagrams

### 1. Normal Operation (Button Press)

```mermaid
sequenceDiagram
    participant UI
    participant Actor
    participant Mapper
    participant UsbHidClient
    participant HostPC

    UI->>Actor: AppEvent::ButtonPressed(id)
    Actor->>Mapper: get_action_sequence(id)
    Mapper-->>Actor: Vec<HidAction>
    loop Action Sequence
        Actor->>UsbHidClient: AppEvent::UsbHidCommand(HID Report)
        Note right of Actor: Or execute Delay
        UsbHidClient->>HostPC: Send HID Report via USB
    end
```

### 2. Unified Configuration Loading at Startup (with `esp-idf-svc`)

```mermaid
sequenceDiagram
    participant Main
    participant EspVfsService
    participant StdFsApi
    participant Mapper
    participant Wifi

    Main->>EspVfsService: Mount LittleFS ("storage" partition at "/littlefs")
    Main->>StdFsApi: File::open("/littlefs/device_config.json")
    alt File Opened
        StdFsApi->>StdFsApi: Read data
        StdFsApi-->>Main: JSON Bytes / Reader
        Main->>Main: Deserialize JSON into DeviceConfiguration
        Main->>Mapper: Mapper::new(config.mappings)
        Main->>Wifi: Wifi::init(config.settings.wifi)
    else File Error (Missing/etc)
        StdFsApi-->>Main: Error
        Main->>Main: Create Default DeviceConfiguration
        Note right of Main: Log Warning
        Main->>StdFsApi: File::create("/littlefs/device_config.json")
        StdFsApi->>StdFsApi: Write Default JSON Bytes
        Main->>Mapper: Mapper::new(default_config.mappings)
        Main->>Wifi: Wifi::init(default_config.settings.wifi)
    end
```

### 3. Configuration Update via WebUSB (with `esp-idf-svc`/`sys`)

```mermaid
graph TD
    subgraph Companion Web App (Browser)
        WebAppUI -->|Generates| JSONConfigString(DeviceConfiguration JSON)
        JSONConfigString -->|Length + Data| WebUSBJS(WebUSB JavaScript API)
    end

    subgraph ESP32 Firmware
        WebUSBHandler(WebUSB Handler Task/Callback) -- Uses --> TinyUSBSys["`esp_idf_svc::sys` WebUSB Functions (unsafe)"]
        WebUSBHandler -->|Validates JSON<br>(serde_json)| WebUSBHandler
        WebUSBHandler -->|Writes File<br>(std::fs)| LittleFS[(LittleFS Partition<br>/littlefs/device_config.json)]
        WebUSBHandler -->|Notifies| Wifi(Wifi Module)
        LittleFS -->|Reads at next boot| Main(Main Module)
        Main -->|Distributes| Mapper
        Main -->|Distributes| Wifi
        TinyUSBCfg(TinyUSB Stack<br>Configured via menuconfig) -- Provides C Funcs --> TinyUSBSys
    end

    WebUSBJS -->|device.transferOut()| USB(USB Cable) --> TinyUSBCfg
```

## Key Architectural Decisions

*   **Unified Configuration:** Storing all user-configurable parameters (mappings, settings) in a single structure (`DeviceConfiguration`) and file (`device_config.json`) simplifies loading and updating.
*   **Decoupling:** `Mapper` handles mappings, `Wifi` handles Wi-Fi, `WebUSB Handler` manages updates. Configuration is loaded centrally in `main` and distributed.
*   **Configuration Format:** JSON remains suitable.
*   **Storage:** LittleFS on internal flash remains suitable.
*   **Communication Protocol:** WebUSB + Length-prefixing remains suitable.
*   **Rust Abstraction:** `esp-idf-svc` is used for safe and idiomatic access to VFS/LittleFS and standard file I/O (`std::fs`, `std::io`). Direct interaction with TinyUSB WebUSB data functions may still require `unsafe` calls to `esp_idf_svc::sys` pending high-level wrappers.
*   **Fallback:** Default configuration generation ensures basic functionality and creates an initial config file.
*   **Applying Settings:** A mechanism (TBD: channels, shared state, signals) is needed for the `WebUSB Handler` to inform relevant modules (like `Wifi`) about potential configuration changes that require action *without* a full device reboot. 