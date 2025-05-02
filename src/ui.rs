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
    // Only proceed if the window still exists
    if let Some(window) = window.upgrade() {
        // Process a new event if one is available
        if let Ok(event) = rx.try_recv() {
            // let mut list_updated = false; // Flag removed

            match event {
                AppEvent::WifiUpdate(status) => {
                    let text = match status {
                        WifiStatus::Initializing => SharedString::from("WiFi: Initializing..."),
                        WifiStatus::Scanning => SharedString::from("WiFi: Scanning..."),
                        WifiStatus::Connecting => SharedString::from("WiFi: Connecting..."),
                        WifiStatus::Connected(ip) => {
                            window.set_wifi_symbol(SharedString::from("🛜"));
                            SharedString::from(&format!("WiFi: Connected to {}", ip))
                        }
                        WifiStatus::Disconnected => {
                            window.set_wifi_symbol(SharedString::from("🛜❌"));
                            SharedString::from("WiFi: Disconnected")
                        }
                        WifiStatus::Error(e) => SharedString::from(&format!("WiFi: Error: {}", e)),
                    };

                    log::info!("{}", text);
                    window.set_status_text(text.clone());
                    status_list_items.push(text);
                    // Update list model immediately after modifying the source Vec
                    let model = (&status_list_items[..]).into();
                    window.set_list_items(model);
                } // Add other AppEvent types here if needed
            }
        }
    }
}
