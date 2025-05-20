use serde::Deserialize;
use serde_json;
use slint::{SharedString, Timer, Weak};
use std::sync::Arc;
use std::time::Duration;

use crate::{http_client::HttpClientPool, ui::window::MainWindow};

#[derive(Deserialize, Debug)]
struct GatusResultItem {
    duration: u64,
}

#[derive(Deserialize, Debug)]
struct GatusResponse {
    results: Vec<GatusResultItem>,
}

fn fetch_and_process_gatus_status(pool: &HttpClientPool, url: &str) -> String {
    match pool.get(url) {
        Ok(json_string) => match serde_json::from_str::<GatusResponse>(&json_string) {
            Ok(parsed_response) => {
                if let Some(first_result) = parsed_response.results.first() {
                    format!("{}ms", first_result.duration / 1_000_000)
                } else {
                    log::warn!("Gatus: No results in the 'results' array.");
                    "N/A (no data)".to_string()
                }
            }
            Err(parse_err) => {
                log::error!("Gatus: Failed to parse JSON response: {}", parse_err);
                "N/A (parse error)".to_string()
            }
        },
        Err(http_err) => {
            log::error!("Gatus: HTTP GET request failed: {}", http_err);
            "N/A (HTTP error)".to_string()
        }
    }
}

pub fn start_gatus_service(
    window_weak: Weak<MainWindow>,
    gatus_url: Arc<String>,
    update_interval: Duration,
    http_pool: Arc<HttpClientPool>,
) {
    let timer = Timer::default();
    timer.start(slint::TimerMode::Repeated, update_interval, move || {
        let window_clone = window_weak.clone();
        let url_clone = Arc::clone(&gatus_url);
        let pool_clone = Arc::clone(&http_pool);

        // No thread spawn needed, pool handles concurrency
        let display_string = fetch_and_process_gatus_status(&pool_clone, &url_clone);

        slint::invoke_from_event_loop(move || {
            if let Some(window) = window_clone.upgrade() {
                // window.set_gatus(SharedString::from(display_string));
            } else {
                log::warn!("Gatus: Slint window was dropped before status could be updated.");
            }
        })
        .expect("Gatus: Failed to schedule UI update on Slint event loop");
    });

    Box::leak(Box::new(timer));
}
