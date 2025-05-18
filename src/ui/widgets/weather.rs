use serde::Deserialize;
use serde_json;
use slint::{SharedString, Timer, Weak};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::{http_client::HttpClient, ui::ui::MainWindow};

// --- Weather API Response Structs (example for OpenWeatherMap-like API) ---
#[derive(Deserialize, Debug)]
struct WeatherMain {
    temp: f32,
    // feels_like: f32,
    // temp_min: f32,
    // temp_max: f32,
    // pressure: i32,
    // humidity: i32,
}

#[derive(Deserialize, Debug)]
struct WeatherInfo {
    // id: i32,
    main: String, // e.g., "Clear", "Clouds", "Rain"
    description: String,
    icon: String, // Icon code, e.g., "01d", "10n"
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    weather: Vec<WeatherInfo>,
    main: WeatherMain,
    // name: String, // City name
}

// --- Helper to map OpenWeatherMap icon codes to simple emojis ---
fn map_weather_icon_code_to_emoji(icon_code: &str) -> SharedString {
    match icon_code {
        "01d" => "☀️".into(),         // clear sky day
        "01n" => "🌙".into(),         // clear sky night
        "02d" => "🌤️".into(),         // few clouds day
        "02n" => "🌥️".into(),         // few clouds night
        "03d" | "03n" => "☁️".into(), // scattered clouds
        "04d" | "04n" => "☁️".into(), // broken clouds (often same as scattered or more cloudy)
        "09d" | "09n" => "🌧️".into(), // shower rain
        "10d" => "🌦️".into(),         // rain day
        "10n" => "🌧️".into(),         // rain night (could use a specific night rain if available)
        "11d" | "11n" => "⛈️".into(), // thunderstorm
        "13d" | "13n" => "❄️".into(), // snow
        "50d" | "50n" => "🌫️".into(), // mist
        _ => "❓".into(),             // Unknown
    }
}

fn fetch_and_process_weather_status(url: &str) -> (SharedString, SharedString, SharedString) {
    match HttpClient::new() {
        Ok(mut client) => match client.get(url, None) {
            Ok(json_string) => {
                match serde_json::from_str::<WeatherResponse>(&json_string) {
                    Ok(parsed_response) => {
                        if let Some(weather_info) = parsed_response.weather.first() {
                            let temp_str = format!("{:.0}°C", parsed_response.main.temp);
                            let desc_str = weather_info.description.to_string();
                            // Capitalize first letter of description
                            let desc_capitalized = if let Some(c) = desc_str.chars().next() {
                                c.to_uppercase().to_string() + &desc_str[1..]
                            } else {
                                desc_str
                            };
                            let icon_emoji = map_weather_icon_code_to_emoji(&weather_info.icon);
                            (icon_emoji, temp_str.into(), desc_capitalized.into())
                        } else {
                            log::warn!("Weather: No weather info in parsed response.");
                            ("❓".into(), "--°C".into(), "N/A (no data)".into())
                        }
                    }
                    Err(parse_err) => {
                        log::error!("Weather: Failed to parse JSON response: {}", parse_err);
                        ("❓".into(), "--°C".into(), "N/A (parse error)".into())
                    }
                }
            }
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
    weather_api_url: Arc<String>,
    update_interval: Duration,
) {
    let timer = Timer::default();
    timer.start(slint::TimerMode::Repeated, update_interval, move || {
        let window_clone = window_weak.clone();
        let url_clone = Arc::clone(&weather_api_url);

        thread::spawn(move || {
            let (icon, temp, desc) = fetch_and_process_weather_status(&url_clone);

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
