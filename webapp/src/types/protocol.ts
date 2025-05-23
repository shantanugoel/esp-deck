// Based on Rust's config.rs and mapper.rs

export type WifiSettings = {
    ssid: string;
    password: string;
};

export type DeviceSettings = {
    wifi?: WifiSettings | null;
    timezone_offset?: number | null;
    api_key?: string | null;
};

// Based on Rust's mapper.rs: ConfigAction
export type ConfigActionKeyPress = {
    type: 'KeyPress';
    keys: string[];
    modifier?: string | null;
};

export type ConfigActionKeyRelease = { type: 'KeyRelease' };

export type ConfigActionMouseMove = {
    type: 'MouseMove';
    dx: number; // i8
    dy: number; // i8
};

export type ConfigActionMousePress = {
    type: 'MousePress';
    button: number; // u8
};

export type ConfigActionMouseRelease = { type: 'MouseRelease' };

export type ConfigActionMouseWheel = {
    type: 'MouseWheel';
    amount: number; // i8
};

export type ConfigActionConsumerPress = {
    type: 'ConsumerPress';
    usage_id: number; // u16
};

export type ConfigActionConsumerRelease = { type: 'ConsumerRelease' };

export type ConfigActionDelay = {
    type: 'Delay';
    ms: number; // u64
};

export type ConfigActionSendString = {
    type: 'SendString';
    keys: string[];
    modifiers: string[];
};

// Forward declaration for recursive type
interface ConfigActionSequenceNode {
    type: 'Sequence';
    actions: ConfigAction[];
}

export type ConfigAction =
    | ConfigActionKeyPress
    | ConfigActionKeyRelease
    | ConfigActionMouseMove
    | ConfigActionMousePress
    | ConfigActionMouseRelease
    | ConfigActionMouseWheel
    | ConfigActionConsumerPress
    | ConfigActionConsumerRelease
    | ConfigActionDelay
    | ConfigActionSendString
    | ConfigActionSequenceNode; // Use the interface here

// Now define the interface if it was forward-declared for clarity, or ensure it's correctly defined if used directly.
// For recursive types, using an interface that then gets included in the union is a common pattern.
export type ConfigActionSequence = ConfigActionSequenceNode;


export type MappingConfiguration = Record<string, ConfigAction[]>; // Corresponds to HashMap<String, Vec<ConfigAction>>

// This type corresponds to Rust's `DeviceConfig` and will be used as `FullDeviceConfig` in stores
export type FullDeviceConfig = {
    settings: DeviceSettings;
    mappings: MappingConfiguration;
    button_names?: Record<number, string> | null; // Corresponds to Option<HashMap<usize, String>>
    widgets?: Record<string, any | null> | null; // Widget configuration
};

// Based on Rust's protocol.rs
export type ProtocolHeader = {
    version: number; // u32, e.g., 0x00010000 for 1.0
    correlationId?: number | null; // Option<u64>
};

// --- Commands (Frontend to Device) ---
// These are the structures for the JSON payload to be sent

// export type GetConfigCommandPayload = {  // OLD, to be removed
//     // type: 'GetConfig';
//     header: ProtocolHeader;
// };

// export type SetConfigCommandPayload = { // OLD, to be removed
//     // type: 'SetConfig';
//     header: ProtocolHeader;
//     config: FullDeviceConfig;
// };

// export type ResetConfigCommandPayload = { // OLD, to be removed
//     // type: 'ResetConfig';
//     header: ProtocolHeader;
// };

// export type RebootCommandPayload = { // OLD, to be removed
//     // type: 'Reboot';
//     header: ProtocolHeader;
// };

// Discriminated union for the command object that will be stringified
// export type CommandPayload = // OLD, to be removed
//     | { type: 'GetConfig'; data: GetConfigCommandPayload }
//     | { type: 'SetConfig'; data: SetConfigCommandPayload }
//     | { type: 'ResetConfig'; data: ResetConfigCommandPayload }
//     | { type: 'Reboot'; data: RebootCommandPayload };

export type GetConfigCommand = {
    type: 'GetConfig';
    header: ProtocolHeader;
};

export type SetConfigCommand = {
    type: 'SetConfig';
    header: ProtocolHeader;
    config: FullDeviceConfig;
};

export type ResetConfigCommand = {
    type: 'ResetConfig';
    header: ProtocolHeader;
};

export type RebootCommand = {
    type: 'Reboot';
    header: ProtocolHeader;
};

// Discriminated union for the command object that will be stringified
export type Command =
    | GetConfigCommand
    | SetConfigCommand
    | ResetConfigCommand
    | RebootCommand;


// --- Responses (Device to Frontend) ---
// These are the structures for the JSON payload received and parsed

export type GetConfigResponsePayload = {
    header: ProtocolHeader;
    config: FullDeviceConfig;
};

export type ErrorResponsePayload = {
    header: ProtocolHeader;
    message: string;
    errorCode: number; // u32
};

export type AckResponsePayload = {
    header: ProtocolHeader;
    message: string;
    success: boolean;
};

// Discriminated union for the parsed response from the device
export type ProtocolResponse =
    | { Config: GetConfigResponsePayload } // Matches Rust's enum Response::Config(GetConfigResponse)
    | { Error: ErrorResponsePayload }     // Matches Rust's enum Response::Error(ErrorResponse)
    | { Ack: AckResponsePayload };        // Matches Rust's enum Response::Ack(AckResponse)


// DeviceInfo for deviceStore.ts (simplified for now)
export type DeviceConnectionInfo = {
    productName?: string; // From WebUSB Device
    firmwareVersion?: string; // Derived from protocol version
    serialNumber?: string; // From WebUSB Device, if available
    // Any other details obtained during WebUSB connection
}; 