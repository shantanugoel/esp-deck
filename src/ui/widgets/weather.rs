use serde::Deserialize;
use serde_json;
use slint::{SharedString, Timer, Weak};
use std::thread;
use std::time::Duration;

use crate::{http_client::HttpClient, ui::window::MainWindow};

#[derive(Deserialize, Debug)]
struct CurrentWeather {
    temperature_2m: f32,
    weather_code: u8,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current: CurrentWeather,
}

fn map_weather_code_to_emoji_and_desc(code: u8) -> (SharedString, SharedString) {
    match code {
        0 => ("☀️".into(), "Clear sky".into()),
        1 | 2 | 3 => ("⛅".into(), "Partly cloudy".into()),
        45 | 48 => ("🌫️".into(), "Fog".into()),
        51 | 53 | 55 => ("🌦️".into(), "Drizzle".into()),
        56 | 57 => ("🌧️".into(), "Freezing Drizzle".into()),
        61 | 63 | 65 => ("🌧️".into(), "Rain".into()),
        66 | 67 => ("🌧️".into(), "Freezing Rain".into()),
        71 | 73 | 75 => ("❄️".into(), "Snow".into()),
        77 => ("❄️".into(), "Snow grains".into()),
        80 | 81 | 82 => ("🌧️".into(), "Rain showers".into()),
        85 | 86 => ("❄️".into(), "Snow showers".into()),
        95 => ("⛈️".into(), "Thunderstorm".into()),
        96 | 99 => ("⛈️".into(), "Thunderstorm w/ hail".into()),
        _ => ("❓".into(), "Unknown".into()),
    }
}

fn fetch_and_process_weather_status(url: &str) -> (SharedString, SharedString, SharedString) {
    match HttpClient::new() {
        Ok(mut client) => match client.get(url, None) {
            Ok(json_string) => match serde_json::from_str::<WeatherResponse>(&json_string) {
                Ok(parsed_response) => {
                    let temp_str = format!("{:.0}°C", parsed_response.current.temperature_2m);
                    let (icon, desc) =
                        map_weather_code_to_emoji_and_desc(parsed_response.current.weather_code);
                    (icon, temp_str.into(), desc)
                }
                Err(parse_err) => {
                    log::error!("Weather: Failed to parse JSON response: {}", parse_err);
                    ("❓".into(), "--°C".into(), "N/A (parse error)".into())
                }
            },
            Err(http_err) => {
                log::error!("Weather: HTTP GET request failed: {}", http_err);
                ("❓".into(), "--°C".into(), "N/A (HTTP error)".into())
            }
        },
        Err(client_init_err) => {
            log::error!("Weather: Failed to create HttpClient: {}", client_init_err);
            ("❓".into(), "--°C".into(), "N/A (client init)".into())
        }
    }
}

pub fn start_weather_service(
    window_weak: Weak<MainWindow>,
    latitude: f32,
    longitude: f32,
    update_interval: Duration,
) {
    let timer = Timer::default();
    timer.start(slint::TimerMode::Repeated, update_interval, move || {
        let window_clone = window_weak.clone();
        // Construct the Open-Meteo API URL with the provided lat/lon
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code",
            latitude, longitude
        );

        thread::spawn(move || {
            let (icon, temp, desc) = fetch_and_process_weather_status(&url);

            slint::invoke_from_event_loop(move || {
                if let Some(window) = window_clone.upgrade() {
                    window.set_weather_icon(icon);
                    window.set_weather_temp(temp);
                    window.set_weather_desc(desc);
                } else {
                    log::warn!("Weather: Slint window was dropped before status could be updated.");
                }
            })
            .expect("Weather: Failed to schedule UI update on Slint event loop");
        });
    });

    Box::leak(Box::new(timer)); // Keep timer alive
}
