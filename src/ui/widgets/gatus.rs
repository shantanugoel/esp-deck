use serde::Deserialize;
use serde_json;
use slint::{SharedString, Weak};
use std::sync::{Arc, Mutex};

use crate::{http_client::HttpClient, ui::ui::MainWindow};

#[derive(Deserialize)]
struct GatusResultItem {
    duration: u64,
}

#[derive(Deserialize)]
struct GatusResponse {
    results: Vec<GatusResultItem>,
}

pub fn gatus_widget_update(window: &Weak<MainWindow>, http_client: Arc<Mutex<HttpClient>>) {
    let mut client = http_client.lock().unwrap();
    let gatus_display_string = match client.get(
        "https://status.shantanugoel.com/api/v1/endpoints/internet_act-status/statuses",
        None,
    ) {
        Ok(json_string) => match serde_json::from_str::<GatusResponse>(&json_string) {
            Ok(parsed_response) => {
                if let Some(first_result) = parsed_response.results.get(0) {
                    format!("{}ms", first_result.duration / 1000000)
                } else {
                    log::warn!("Gatus response had no results in the 'results' array.");
                    "N/A (no data)".to_string()
                }
            }
            Err(parse_err) => {
                log::error!("Failed to parse Gatus JSON response: {}", parse_err);
                "N/A (parse error)".to_string()
            }
        },
        Err(http_err) => {
            log::error!("Failed to get gatus status via HTTP: {}", http_err);
            "N/A (HTTP error)".to_string()
        }
    };
    window
        .upgrade()
        .unwrap()
        .set_gatus(SharedString::from(gatus_display_string));
}
