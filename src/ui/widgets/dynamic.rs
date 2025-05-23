use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use crate::{
    config::{WidgetItemConfig, WidgetKindConfig},
    http_client::HttpClientPool,
    ui::window::{MainWindow, WidgetItem, WidgetKind},
};
use anyhow::Result;
use slint::{
    Image, Model, ModelExt, ModelRc, Rgb8Pixel, SharedPixelBuffer, SharedString, Timer, TimerMode,
    VecModel, Weak,
};

pub fn start_widget_service(
    window: Weak<MainWindow>,
    http_pool: Arc<HttpClientPool>,
    widgets: Option<HashMap<usize, WidgetItemConfig>>,
) {
    if let Some(widgets) = widgets {
        let model = Rc::new(VecModel::<WidgetItem>::from(Vec::new()));
        for (id, widget) in widgets {
            let mut widget_item = WidgetItem {
                title: SharedString::from(widget.title.clone()),
                id: id as i32,
                ..Default::default()
            };
            match widget.kind {
                WidgetKindConfig::Text(_, _) => {
                    widget_item.value.kind = WidgetKind::Text;
                }
                WidgetKindConfig::Image(_) => {
                    widget_item.value.kind = WidgetKind::Image;
                }
            }
            model.push(widget_item);
            let timer = Timer::default();
            let (timer_mode, update_interval) = match widget.update_interval_seconds {
                0 => (TimerMode::SingleShot, 5),
                _ => (TimerMode::Repeated, widget.update_interval_seconds),
            };
            let window_clone = window.clone();
            let http_pool_clone = http_pool.clone();

            // If the update interval is too long, let's do a first display soon
            if update_interval > 30 {
                let window_clone_2 = window_clone.clone();
                let widget_clone = widget.clone();
                let http_pool_clone_2 = http_pool_clone.clone();
                let timer_first_tick = Timer::default();
                timer_first_tick.start(TimerMode::SingleShot, Duration::from_secs(15), move || {
                    display_widget(
                        &window_clone_2,
                        id as i32,
                        &widget_clone,
                        &http_pool_clone_2,
                    );
                });
                Box::leak(Box::new(timer_first_tick));
            }

            timer.start(
                timer_mode,
                Duration::from_secs(update_interval),
                move || {
                    display_widget(&window_clone, id as i32, &widget, &http_pool_clone);
                },
            );
            Box::leak(Box::new(timer));
        }
        window
            .upgrade()
            .unwrap()
            .set_dashboard_items(ModelRc::from(model));
        let count = window.upgrade().unwrap().get_dashboard_items().row_count();
        log::info!("Dashboard items count: {}", count);
    }
}

fn display_widget(
    window: &Weak<MainWindow>,
    id: i32,
    widget: &WidgetItemConfig,
    http_pool: &HttpClientPool,
) {
    log::info!("Displaying widget: {}", widget.title);
    let window = window.upgrade().unwrap();
    let model = window
        .get_dashboard_items()
        .filter(move |item| item.id == id);
    let mut widget_item = model.row_data(0).unwrap();
    match widget.kind.clone() {
        WidgetKindConfig::Text(url, path) => {
            let text = fetch_and_process_text(http_pool, &url, &path);
            if let Ok(text) = text {
                widget_item.value.value_string = text;
            } else {
                log::error!("Failed to fetch text: {}", text.err().unwrap());
            }
        }
        WidgetKindConfig::Image(value) => {
            let image = fetch_and_process_image(http_pool, &value);
            if let Ok(image) = image {
                widget_item.value.value_image = Image::from_rgb8(image);
            } else {
                log::error!("Failed to fetch image: {}", image.err().unwrap());
            }
        }
    }
    model.set_row_data(0, widget_item);
}

fn fetch_and_process_text(
    pool: &HttpClientPool,
    url: &str,
    path: &Option<String>,
) -> Result<SharedString> {
    let text = pool.get(url)?;

    let processed_text = if let Some(path) = path {
        let json = serde_json::from_str::<serde_json::Value>(&text)?;
        if let Some(value) = json.pointer(path) {
            value.to_string()
        } else {
            log::info!("No value found at path: {}", path);
            text
        }
    } else {
        text
    };

    let final_text = if processed_text.len() > 100 {
        processed_text[0..100].to_string()
    } else {
        processed_text
    };

    Ok(SharedString::from(final_text))
}

fn fetch_and_process_image(
    pool: &HttpClientPool,
    url: &str,
) -> Result<SharedPixelBuffer<Rgb8Pixel>> {
    let image_data_result = pool.get_bytes(url);

    match image_data_result {
        Ok(bytes) => {
            log::info!("Image data size: {} bytes", bytes.len());
            match image::load_from_memory(bytes.as_slice()) {
                Ok(image) => {
                    log::info!("Free heap after load_from_memory (Ok): {}", unsafe {
                        esp_idf_svc::sys::esp_get_free_heap_size()
                    });
                    log::info!(
                        "Original image dimensions: {}x{}",
                        image.width(),
                        image.height()
                    );
                    let image = if image.width() > 100 || image.height() > 100 {
                        log::info!("Resizing image to 100x100");
                        image.resize(100, 100, image::imageops::FilterType::Nearest)
                    } else {
                        image
                    };
                    log::info!("Free heap after resize: {}", unsafe {
                        esp_idf_svc::sys::esp_get_free_heap_size()
                    });
                    let shared_image = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                        image.to_rgb8().into_raw().as_slice(),
                        image.width(),
                        image.height(),
                    );
                    Ok(shared_image)
                }
                Err(e) => {
                    log::error!("Failed to decode image (load_from_memory error): {}", e);
                    Err(anyhow::anyhow!("Failed to decode image: {}", e))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get_bytes: {}", e);
            Err(e)
        }
    }
}
