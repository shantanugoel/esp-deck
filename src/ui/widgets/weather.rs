use serde::Deserialize;
use serde_json;
use slint::{SharedString, Timer, Weak};
use std::sync::Arc;
use std::time::Duration;

use crate::{http_client::HttpClientPool, ui::window::MainWindow};

#[derive(Deserialize, Debug)]
struct CurrentWeather {
    temperature_2m: f32,
    weather_code: u8,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current: CurrentWeather,
}

fn map_weather_code_to_emoji(code: u8) -> SharedString {
    match code {
        0 => "☀️".into(),
        1..=3 => "⛅".into(),
        45 | 48 => "🌫️".into(),
        51 | 53 | 55 => "🌦️".into(),
        56 | 57 => "🌧️".into(),
        61 | 63 | 65 => "🌧️".into(),
        66 | 67 => "🌧️".into(),
        71 | 73 | 75 => "❄️".into(),
        77 => "❄️".into(),
        80..=82 => "🌧️".into(),
        85 | 86 => "❄️".into(),
        95 => "⛈️".into(),
        96 | 99 => "⛈️".into(),
        _ => "❓".into(),
    }
}

fn fetch_and_process_weather_status(
    pool: &HttpClientPool,
    url: &str,
) -> (SharedString, SharedString) {
    match pool.get(url) {
        Ok(json_string) => match serde_json::from_str::<WeatherResponse>(&json_string) {
            Ok(parsed_response) => {
                let temp_str = format!("{:.0}°C", parsed_response.current.temperature_2m);
                let icon = map_weather_code_to_emoji(parsed_response.current.weather_code);
                (icon, temp_str.into())
            }
            Err(parse_err) => {
                log::error!("Weather: Failed to parse JSON response: {}", parse_err);
                ("❓".into(), "--°C".into())
            }
        },
        Err(http_err) => {
            log::error!("Weather: HTTP GET request failed: {}", http_err);
            ("❓".into(), "--°C".into())
        }
    }
}

pub fn start_weather_service(
    window_weak: Weak<MainWindow>,
    latitude: f32,
    longitude: f32,
    update_interval: Duration,
    http_pool: Arc<HttpClientPool>,
) {
    let timer = Timer::default();
    timer.start(slint::TimerMode::Repeated, update_interval, move || {
        let window_clone = window_weak.clone();
        let pool_clone = Arc::clone(&http_pool);
        // Construct the Open-Meteo API URL with the provided lat/lon
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code",
            latitude, longitude
        );

        // No thread spawn needed, pool handles concurrency
        let (icon, temp) = fetch_and_process_weather_status(&pool_clone, &url);

        slint::invoke_from_event_loop(move || {
            if let Some(window) = window_clone.upgrade() {
                window.set_weather_icon(icon);
                window.set_weather_temp(temp);
            } else {
                log::warn!("Weather: Slint window was dropped before status could be updated.");
            }
        })
        .expect("Weather: Failed to schedule UI update on Slint event loop");
    });

    Box::leak(Box::new(timer)); // Keep timer alive
}
