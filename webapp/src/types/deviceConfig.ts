// Corresponds to Rust's enum WidgetKindConfig
export type WidgetKindConfigFE =
    | { Text: [string, string | null] } // [url, jsonPointer | null]
    | { Image: string };             // [url]

// Corresponds to Rust's struct WidgetItemConfig
export type WidgetItemConfigFE = {
    title: string;
    kind: WidgetKindConfigFE;
    update_interval_seconds: number;
};

// For sending updates to backend: key is widget ID (number),
// value is WidgetItemConfigFE for add/update, or null for delete.
export type WidgetsConfigPayload = Record<number, WidgetItemConfigFE | null>;

// For representing the state of widgets in the frontend, always with full config items.
export type WidgetsState = Record<number, WidgetItemConfigFE>;

// Based on src/config.rs DeviceConfig and focusing on what frontend needs
export type DeviceConfigFE = {
    settings?: {
        wifi?: {
            ssid: string;
            password: string;
        };
        timezone_offset?: number;
        api_key?: string | null;
    };
    mappings?: Record<string, unknown[]>; // Define more specifically when mappings are implemented
    button_names?: Record<number, string>;
    widgets?: WidgetsState; // Received from backend, keys are usize, values are WidgetItemConfig
};

// For the widget add/edit form
export type WidgetFormState = {
    id: number | null; // null for new, number for existing being edited
    title: string;
    type: 'Text' | 'Image';
    url: string;
    jsonPointer: string | null;
    update_interval_seconds: number;
    isJson: boolean; // UI helper for Text widget form
}; 