use anyhow::Result;
use esp_idf_svc::hal::i2c::I2cDriver;
use slint::{SharedString, Weak};
use std::{sync::mpsc::Receiver, time::Duration};

use crate::{
    bsp::slint_platform,
    events::{AppEvent, WifiStatus},
};

slint::include_modules!();
pub struct Window;

impl Window {
    pub fn init(touch_i2c: I2cDriver<'static>, rx: Receiver<AppEvent>) -> Result<()> {
        slint_platform::init(touch_i2c);
        let window = MainWindow::new()
            .map_err(|e| anyhow::anyhow!("Failed to create main window: {}", e))?;

        install_test_callback(&window);

        let weak_window = window.as_weak();
        let timer = slint::Timer::default();
        let mut status_list_items: Vec<SharedString> = Vec::new();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(500),
            move || {
                handle_events(&weak_window, &rx, &mut status_list_items);
            },
        );

        window
            .run()
            .map_err(|e| anyhow::anyhow!("Failed to run main window: {}", e))?;

        Ok(())
    }
}

fn install_test_callback(window: &MainWindow) {
    let _ = window.as_weak();
    window.on_update_fact(move || {
        log::info!("Test callback");
    });
}

fn handle_events(
    window: &Weak<MainWindow>,
    rx: &Receiver<AppEvent>,
    status_list_items: &mut Vec<SharedString>,
) {
    log::info!("Handling events");
    let window = window.upgrade().unwrap();
    window.set_list_items(status_list_items.into());
    if let Ok(event) = rx.try_recv() {
        match event {
            AppEvent::WifiUpdate(status) => match status {
                WifiStatus::Initializing => {
                    let text = SharedString::from("WiFi: Initializing...");
                    window.set_status_text(text.clone());
                    status_list_items.push(text);
                    log::info!("Initializing...");
                }
                WifiStatus::Scanning => {
                    let text = SharedString::from("WiFi: Scanning...");
                    window.set_status_text(text.clone());
                    status_list_items.push(text);
                    log::info!("Scanning...");
                }
                WifiStatus::Connecting => {
                    let text = SharedString::from("WiFi: Connecting...");
                    window.set_status_text(text.clone());
                    status_list_items.push(text);
                    log::info!("Connecting...");
                }
                WifiStatus::Connected(ip) => {
                    let text = SharedString::from(&format!("WiFi: Connected to {}", ip));
                    window.set_status_text(text.clone());
                    window.set_wifi_symbol(SharedString::from("🛜"));
                    status_list_items.push(text);
                    log::info!("Connected to {}", ip);
                }
                WifiStatus::Disconnected => {
                    let text = SharedString::from("WiFi: Disconnected");
                    window.set_status_text(text.clone());
                    status_list_items.push(text);
                    log::info!("Disconnected");
                }
                WifiStatus::Error(e) => {
                    let text = SharedString::from(&format!("WiFi: Error: {}", e));
                    window.set_status_text(text.clone());
                    status_list_items.push(text);
                    log::info!("Error: {}", e);
                }
            },
        }
    }
}
