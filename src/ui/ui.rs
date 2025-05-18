use anyhow::Result;
use chrono::{DateTime, FixedOffset, Utc};
use esp_idf_svc::hal::i2c::I2cDriver;
use slint::{SharedString, Weak};
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
    events::{AppEvent, TimeStatus, UsbStatus, WifiStatus},
};

use super::widgets::gatus::start_gatus_service;
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

        let gatus_url_arc = Arc::new(
            "https://status.shantanugoel.com/api/v1/endpoints/internet_act-status/statuses"
                .to_string(),
        );
        let gatus_update_interval = Duration::from_secs(30);
        start_gatus_service(window.as_weak(), gatus_url_arc, gatus_update_interval);

        let weather_api_url_arc = Arc::new(
            "https://api.openweathermap.org/data/2.5/weather?q=London&appid=YOUR_OPENWEATHERMAP_API_KEY&units=metric"
                .to_string(),
        );
        let weather_update_interval = Duration::from_secs(10 * 60);
        start_weather_service(
            window.as_weak(),
            weather_api_url_arc,
            weather_update_interval,
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
