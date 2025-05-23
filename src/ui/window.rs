use anyhow::Result;
use chrono::{DateTime, FixedOffset, Utc};
use esp_idf_svc::hal::i2c::I2cDriver;
use slint::{Color, SharedString, Weak};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use crate::{
    bsp::slint_platform,
    config::WidgetItemConfig,
    events::{AppEvent, TimeStatus, UsbStatus, WifiStatus},
    http_client::HttpClientPool,
};

use super::widgets::weather::start_weather_service;

slint::include_modules!();
pub struct Window;

impl Window {
    pub fn init(
        touch_i2c: I2cDriver<'static>,
        rx: Receiver<AppEvent>,
        actor_tx: Sender<AppEvent>,
        tz_offset: f32,
        button_names: Option<HashMap<usize, String>>,
        http_pool: Arc<HttpClientPool>,
        widgets: Option<HashMap<usize, WidgetItemConfig>>,
    ) -> Result<()> {
        slint_platform::init(touch_i2c);
        let window = MainWindow::new()
            .map_err(|e| anyhow::anyhow!("Failed to create main window: {}", e))?;

        set_button_names(&window, button_names);
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

        let weak_window_time_date_updates = window.as_weak();
        let timer_time_date_updates = slint::Timer::default();
        timer_time_date_updates.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || update_time_and_date(&weak_window_time_date_updates, tz_offset),
        );

        let button_actor_tx = actor_tx.clone();
        window.on_button_pressed(move |button_id: i32| {
            log::info!("Button {} pressed in UI! Sending to Actor.", button_id);
            let _ = button_actor_tx.send(AppEvent::ButtonPressed(button_id));
        });

        super::widgets::dynamic::start_widget_service(window.as_weak(), http_pool.clone(), widgets);
        super::widgets::server::start_server_widget_service(window.as_weak());

        let weather_update_interval = Duration::from_secs(10 * 60);
        start_weather_service(
            window.as_weak(),
            12.9716, // Harcode BLR for now
            77.5946, // Harcode BLR for now
            weather_update_interval,
            http_pool,
        );

        window
            .run()
            .map_err(|e| anyhow::anyhow!("Failed to run main window: {}", e))?;

        Ok(())
    }
}

pub fn set_button_names(window: &MainWindow, button_names: Option<HashMap<usize, String>>) {
    let mut names: Vec<SharedString> = Vec::with_capacity(16);
    for i in 0..16 {
        let name = button_names
            .as_ref()
            .and_then(|map| map.get(&i))
            .map(|s| {
                if s.len() > 20 {
                    SharedString::from(&s[..20])
                } else {
                    SharedString::from(s)
                }
            })
            .unwrap_or_else(|| SharedString::from(format!("Button {}", i + 1)));
        names.push(name);
    }
    window.set_button_names(names.as_slice().into());
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
    if let Some(window) = window.upgrade() {
        if let Ok(event) = rx.try_recv() {
            let text = match event {
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
                    Some(text)
                }
                AppEvent::TimeUpdate(status) => {
                    let text = match status {
                        TimeStatus::Initializing => SharedString::from("Time: Initializing..."),
                        TimeStatus::Synced => SharedString::from("Time: Synced"),
                        TimeStatus::Error(e) => SharedString::from(&format!("Time: Error: {}", e)),
                    };
                    Some(text)
                }
                AppEvent::UsbUpdate(status) => {
                    let text = match status {
                        UsbStatus::Initialized => SharedString::from("USB: Initialized"),
                        UsbStatus::Connected => SharedString::from("USB: Connected"),
                        UsbStatus::Disconnected => SharedString::from("USB: Disconnected"),
                        UsbStatus::Suspended => SharedString::from("USB: Suspended"),
                        UsbStatus::Error(e) => SharedString::from(&format!("USB: Error: {}", e)),
                    };
                    Some(text)
                }
                AppEvent::UserStatusUpdate(user_status) => {
                    window.set_user_status_text(SharedString::from(&user_status.text));
                    if let Some(bgcolor) = user_status.bgcolor {
                        window.set_user_status_bgcolor(Color::from_rgb_u8(
                            bgcolor[0], bgcolor[1], bgcolor[2],
                        ));
                    }
                    // Return None since we don't want to add this to the UI logs
                    None
                }
                AppEvent::HttpServerUpdate(status) => {
                    // Return None since we don't want to add this to the UI logs
                    Some(SharedString::from(&status))
                }
                AppEvent::ServerWidgetUpdate(data) => match data.id {
                    1 => {
                        window.set_server_widget_2_title(SharedString::from(&data.title));
                        window.set_server_widget_2_value(SharedString::from(&data.value));
                        Some(SharedString::from("Updated Server Widget 1"))
                    }
                    2 => {
                        window.set_server_widget_3_title(SharedString::from(&data.title));
                        window.set_server_widget_3_value(SharedString::from(&data.value));
                        Some(SharedString::from("Updated Server Widget 2"))
                    }
                    _ => {
                        log::error!("Unknown server widget id: {}", data.id);
                        Some(SharedString::from(
                            "Received request for unknown server widget id",
                        ))
                    }
                },
                _ => {
                    log::info!("Unknown event: {:?}", event);
                    None
                }
            };

            if let Some(text) = text {
                log::info!("{}", text);
                window.set_status_text(text.clone());
                status_list_items.push(text);
                let model = (&status_list_items[..]).into();
                window.set_list_items(model);
            }
        }
    }
}

fn update_time_and_date(window: &Weak<MainWindow>, tz_offset: f32) {
    let now_utc: DateTime<Utc> = std::time::SystemTime::now().into();
    let seconds_offset = (tz_offset * 3600.0) as i32;
    let fixed_offset = match FixedOffset::east_opt(seconds_offset) {
        Some(offset) => offset,
        None => {
            log::error!("Invalid timezone offset: {}", tz_offset);
            FixedOffset::east_opt(0).unwrap()
        }
    };
    let local_time = now_utc.with_timezone(&fixed_offset);

    let time_str_detailed = SharedString::from(local_time.format("%H:%M:%S").to_string());
    let date_str_detailed =
        SharedString::from(local_time.format("%a, %b %d").to_string().to_uppercase());

    if let Some(window_strong) = window.upgrade() {
        if window_strong.get_current_time() != time_str_detailed {
            window_strong.set_current_time(time_str_detailed);
        }
        if window_strong.get_current_date() != date_str_detailed {
            window_strong.set_current_date(date_str_detailed);
        }
    } else {
        log::error!("Failed to upgrade window weak reference in update_time_and_date");
    }
}
