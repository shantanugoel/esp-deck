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
    // Don't log on every tick unless debugging, can be noisy
    // log::info!("Handling events");

    // Only proceed if the window still exists
    if let Some(window) = window.upgrade() {
        // Process a new event if one is available
        if let Ok(event) = rx.try_recv() {
            let mut list_updated = false; // Flag to track if we need to update the UI list

            match event {
                AppEvent::WifiUpdate(status) => {
                    let text = match status {
                        WifiStatus::Initializing => SharedString::from("WiFi: Initializing..."),
                        WifiStatus::Scanning => SharedString::from("WiFi: Scanning..."),
                        WifiStatus::Connecting => SharedString::from("WiFi: Connecting..."),
                        WifiStatus::Connected(ip) => {
                            // Also update the symbol when connected
                            window.set_wifi_symbol(SharedString::from("🛜"));
                            SharedString::from(&format!("WiFi: Connected to {}", ip))
                        }
                        WifiStatus::Disconnected => {
                            // Update symbol when disconnected
                            window.set_wifi_symbol(SharedString::from("🛜❌")); // Assuming this is your disconnected symbol
                            SharedString::from("WiFi: Disconnected")
                        }
                        WifiStatus::Error(e) => SharedString::from(&format!("WiFi: Error: {}", e)),
                    };

                    log::info!("{}", text); // Log the event text
                    window.set_status_text(text.clone()); // Update the status bar text
                    status_list_items.push(text); // Add to our local history
                    list_updated = true; // Mark that the list changed
                } // Add other AppEvent types here if needed
            }

            // If we added an item, update the Slint model
            if list_updated {
                // Create a ModelRc from a slice of the current Vec
                let model = (&status_list_items[..]).into();
                window.set_list_items(model);
            }
        }
        // Removed: Incorrect/inefficient set_list_items call was here
    }
}
