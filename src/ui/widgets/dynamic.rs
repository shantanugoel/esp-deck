use std::{collections::HashMap, rc::Rc, sync::Arc, time::Duration};

use crate::{
    config::{WidgetItemConfig, WidgetKindConfig},
    http_client::HttpClientPool,
    ui::window::{MainWindow, WidgetItem, WidgetItemValue, WidgetKind},
};
use anyhow::Result;
use slint::{
    Image, Model, ModelRc, Rgb8Pixel, SharedPixelBuffer, SharedString, Timer, TimerMode, VecModel,
    Weak,
};

#[derive(Debug, Clone, Copy)]
enum Kind {
    Text(&'static str),
    Image(&'static str),
}

#[derive(Debug, Clone, Copy)]
struct WidgetItemData {
    title: &'static str,
    kind: Kind,
}

pub fn start_widget_service(
    window: Weak<MainWindow>,
    http_pool: Arc<HttpClientPool>,
    widgets: Option<HashMap<usize, WidgetItemConfig>>,
) {
    if let Some(widgets) = widgets {
        let model = Rc::new(VecModel::<WidgetItem>::from(Vec::new()));
        for (key, widget) in widgets {
            let mut widget_item = WidgetItem::default();
            widget_item.title = SharedString::from(widget.title.clone());
            match widget.kind {
                WidgetKindConfig::Text(_) => {
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
            timer.start(
                timer_mode,
                Duration::from_secs(update_interval),
                move || {
                    display_widget(&window_clone, &widget, &http_pool_clone);
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
    widget: &WidgetItemConfig,
    http_pool: &HttpClientPool,
) {
    log::info!("Displaying widget: {}", widget.title);
    let window = window.upgrade().unwrap();
    let model = window.get_dashboard_items();
    let mut widget_item = model.row_data(0).unwrap();
    match widget.kind.clone() {
        WidgetKindConfig::Text(value) => {
            widget_item.value.value_string = SharedString::from(value);
        }
        WidgetKindConfig::Image(value) => {
            let image = fetch_and_process_image(&http_pool, &value);
            if let Ok(image) = image {
                widget_item.value.value_image = Image::from_rgb8(image);
            }
        }
    }
    model.set_row_data(0, widget_item);
}

// pub fn start_dynamic_service2(window: Weak<MainWindow>, http_pool: Arc<HttpClientPool>) {
//     // Wait till wifi is connected
//     std::thread::sleep(std::time::Duration::from_millis(5000));
//     log::info!("Free heap before: {}", unsafe {
//         esp_idf_svc::sys::esp_get_minimum_free_heap_size()
//     });
//     let model = Rc::new(VecModel::<WidgetItem>::from(Vec::new()));

//     for item in WIDGET_ITEMS {
//         let mut widget_item = WidgetItem::default();
//         widget_item.title = SharedString::from(item.title);
//         match item.kind {
//             Kind::Text(text) => {
//                 widget_item.value.kind = WidgetKind::Text;
//                 widget_item.value.value_string = SharedString::from(text);
//             }
//             Kind::Image(image) => {
//                 widget_item.value.kind = WidgetKind::Image;
//                 let image = fetch_and_process_image(&http_pool, image);
//                 if let Ok(image) = image {
//                     widget_item.value.value_image = Image::from_rgb8(image);
//                 } else {
//                     log::error!("Failed to fetch image: {}", image.err().unwrap());
//                 }
//             }
//         }
//         model.push(widget_item);
//     }
//     log::info!("Free heap middle: {}", unsafe {
//         esp_idf_svc::sys::esp_get_minimum_free_heap_size()
//     });

//     window
//         .upgrade()
//         .unwrap()
//         .set_dashboard_items(ModelRc::from(model));
//     log::info!("Free heap after: {}", unsafe {
//         esp_idf_svc::sys::esp_get_minimum_free_heap_size()
//     });
// }

fn fetch_and_process_image(
    pool: &HttpClientPool,
    url: &str,
) -> Result<SharedPixelBuffer<Rgb8Pixel>> {
    match image::load_from_memory(pool.get_bytes(url)?.as_slice()) {
        Ok(image) => {
            // If image is larger than 100x100, resize it to 100x100
            let image = if image.width() > 100 || image.height() > 100 {
                image.resize(100, 100, image::imageops::FilterType::Nearest)
            } else {
                image
            };
            log::info!("Image size: {}x{}", image.width(), image.height());
            let shared_image = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                image.to_rgb8().into_raw().as_slice(),
                image.width(),
                image.height(),
            );
            Ok(shared_image)
        }
        Err(e) => Err(anyhow::anyhow!("Failed to decode image: {}", e)),
    }
}
