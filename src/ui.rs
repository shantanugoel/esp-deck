use anyhow::Result;
use chrono::{DateTime, FixedOffset, Utc};
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
    pub fn init(
        touch_i2c: I2cDriver<'static>,
        rx: Receiver<AppEvent>,
        tz_offset: f32,
    ) -> Result<()> {
        slint_platform::init(touch_i2c);
        let window = MainWindow::new()
            .map_err(|e| anyhow::anyhow!("Failed to create main window: {}", e))?;

        install_test_callback(&window);

        let weak_window_status_updates = window.as_weak();
        let timer_status_updates = slint::Timer::default();
        let mut status_list_items: Vec<SharedString> = Vec::new();
        timer_status_updates.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(500),
            move || {
                handle_events(&weak_window_status_updates, &rx, &mut status_list_items);
            },
        );

        let weak_window_time_updates = window.as_weak();
        let timer_time_updates = slint::Timer::default();
        timer_time_updates.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(2000),
            move || update_time(&weak_window_time_updates, tz_offset),
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

fn update_time(window: &Weak<MainWindow>, tz_offset: f32) {
    let time: DateTime<Utc> = std::time::SystemTime::now().into();
    let seconds_offset = tz_offset * 3600.0;
    let fixed_offset = FixedOffset::east_opt(seconds_offset as i32).unwrap();
    let time_str = time
        .with_timezone(&fixed_offset)
        .format("%H:%M")
        .to_string();
    window
        .upgrade()
        .unwrap()
        .set_current_time(SharedString::from(time_str));
}
