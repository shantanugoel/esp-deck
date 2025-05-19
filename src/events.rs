use crate::{
    bsp::usb_desc::{ConsumerReport, KeyboardReport, MouseReport},
    mapper::MappingConfiguration,
};
use std::net::Ipv4Addr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum WifiStatus {
    Initializing,
    Scanning,
    Connecting,
    Connected(Ipv4Addr),
    Disconnected,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum TimeStatus {
    Initializing,
    Synced,
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub enum UsbHidCommand {
    SendKeyboard(KeyboardReport),
    SendMouse(MouseReport),
    SendConsumer(ConsumerReport),
}

#[derive(Debug, Clone)]
pub enum UsbStatus {
    Initialized,
    Connected,
    Disconnected,
    Suspended,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    WifiUpdate(WifiStatus),
    TimeUpdate(TimeStatus),
    UsbUpdate(UsbStatus),
    UsbHidCommand(UsbHidCommand),
    ButtonPressed(i32),
    MappingUpdated(MappingConfiguration),
    UserStatusUpdate(String),
    HttpServerUpdate(String),
}

// Represents a single primitive HID action or delay
#[derive(Debug, Clone)]
pub enum HidAction {
    KeyPress(u8, [u8; 6]), // modifier, up to 6 keycodes
    KeyRelease,            // Releases all keys/modifiers
    MouseMove(i8, i8),     // dx, dy
    MousePress(u8),        // buttons bitmask
    MouseRelease,          // Releases all buttons
    MouseWheel(i8),        // wheel movement
    ConsumerPress(u16),    // usage_id
    ConsumerRelease,       // Releases consumer control
    Delay(Duration),       // Pause execution
}
