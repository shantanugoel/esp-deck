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
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(500),
            move || {
                handle_events(&weak_window, &rx);
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

fn handle_events(window: &Weak<MainWindow>, rx: &Receiver<AppEvent>) {
    log::info!("Handling events");
    let window = window.upgrade().unwrap();
    if let Ok(event) = rx.try_recv() {
        match event {
            AppEvent::WifiUpdate(status) => match status {
                WifiStatus::Initializing => {
                    window.set_status_text(SharedString::from("Initializing..."));
                    log::info!("Initializing...");
                }
                WifiStatus::Scanning => {
                    window.set_status_text(SharedString::from("Scanning..."));
                    log::info!("Scanning...");
                }
                WifiStatus::Connecting => {
                    window.set_status_text(SharedString::from("Connecting..."));
                    log::info!("Connecting...");
                }
                WifiStatus::Connected(ip) => {
                    window.set_status_text(SharedString::from(&format!("Connected to {}", ip)));
                    window.set_wifi_symbol(SharedString::from("🛜"));
                    log::info!("Connected to {}", ip);
                }
                WifiStatus::Disconnected => {
                    window.set_status_text(SharedString::from("Disconnected"));
                    log::info!("Disconnected");
                }
                WifiStatus::Error(e) => {
                    window.set_status_text(SharedString::from(&format!("Error: {}", e)));
                    log::info!("Error: {}", e);
                }
            },
        }
    }
}
