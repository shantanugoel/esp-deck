use std::net::Ipv4Addr;

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

#[derive(Debug, Clone)]
pub enum AppEvent {
    WifiUpdate(WifiStatus),
    TimeUpdate(TimeStatus),
}
