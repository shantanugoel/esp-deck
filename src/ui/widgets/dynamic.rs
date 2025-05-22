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
            let mut widget_item = WidgetItem::default();
            widget_item.title = SharedString::from(widget.title.clone());
            widget_item.id = id as i32;
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
        WidgetKindConfig::Text(value) => {
            let text = fetch_and_process_text(&http_pool, &value);
            if let Ok(text) = text {
                widget_item.value.value_string = text;
            } else {
                log::error!("Failed to fetch text: {}", text.err().unwrap());
            }
        }
        WidgetKindConfig::Image(value) => {
            let image = fetch_and_process_image(&http_pool, &value);
            if let Ok(image) = image {
                widget_item.value.value_image = Image::from_rgb8(image);
            } else {
                log::error!("Failed to fetch image: {}", image.err().unwrap());
            }
        }
    }
    model.set_row_data(0, widget_item);
}

fn fetch_and_process_text(pool: &HttpClientPool, url: &str) -> Result<SharedString> {
    let text = pool.get(url)?;

    //TODO: Process text
    Ok(SharedString::from(text))
}

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
