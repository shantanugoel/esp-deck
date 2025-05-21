use std::{rc::Rc, sync::Arc};

use crate::{
    http_client::HttpClientPool,
    ui::window::{MainWindow, WidgetItem, WidgetItemValue, WidgetKind},
};
use anyhow::Result;
use esp_idf_svc::sys::{
    esp_jpeg_decode, esp_jpeg_get_image_info, esp_jpeg_image_cfg_t,
    esp_jpeg_image_format_t_JPEG_IMAGE_FORMAT_RGB888, esp_jpeg_image_output_t,
};
use slint::{Image, ModelRc, Rgb8Pixel, SharedPixelBuffer, SharedString, VecModel, Weak};

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

static WIDGET_ITEMS: [WidgetItemData; 4] = [
    WidgetItemData {
        title: "Hello",
        kind: Kind::Text("Hello"),
    },
    WidgetItemData {
        title: "Hello2",
        kind: Kind::Image("https://shantanugoel.com/img/avatar.jpg"),
    },
    WidgetItemData {
        title: "Hello3",
        kind: Kind::Image("https://www.gstatic.com/webp/gallery/1.webp"),
    },
    WidgetItemData {
        title: "Hello4",
        kind: Kind::Image("https://www.gstatic.com/webp/gallery/2.png"),
    },
];

pub fn start_dynamic_service(window: Weak<MainWindow>, http_pool: Arc<HttpClientPool>) {
    // Wait till wifi is connected
    std::thread::sleep(std::time::Duration::from_millis(5000));
    log::info!("Free heap before: {}", unsafe {
        esp_idf_svc::sys::esp_get_minimum_free_heap_size()
    });
    let model = Rc::new(VecModel::<WidgetItem>::from(Vec::new()));

    for item in WIDGET_ITEMS {
        let mut widget_item = WidgetItem::default();
        widget_item.title = SharedString::from(item.title);
        match item.kind {
            Kind::Text(text) => {
                widget_item.value.kind = WidgetKind::Text;
                widget_item.value.value_string = SharedString::from(text);
            }
            Kind::Image(image) => {
                widget_item.value.kind = WidgetKind::Image;
                let image = fetch_and_process_image(&http_pool, image);
                if let Ok(image) = image {
                    widget_item.value.value_image = Image::from_rgb8(image);
                } else {
                    log::error!("Failed to fetch image: {}", image.err().unwrap());
                }
            }
        }
        model.push(widget_item);
    }
    log::info!("Free heap middle: {}", unsafe {
        esp_idf_svc::sys::esp_get_minimum_free_heap_size()
    });

    window
        .upgrade()
        .unwrap()
        .set_dashboard_items(ModelRc::from(model));
    log::info!("Free heap after: {}", unsafe {
        esp_idf_svc::sys::esp_get_minimum_free_heap_size()
    });
}

fn fetch_and_process_image(
    pool: &HttpClientPool,
    url: &str,
) -> Result<SharedPixelBuffer<Rgb8Pixel>> {
    // match ImageReader::new(Cursor::new(pool.get_bytes(url)?))
    //     .with_guessed_format()
    //     .map_err(|e| anyhow::anyhow!("Failed to guess image format: {}", e))?
    //     .decode()
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

fn fetch_and_process_image2(
    pool: &HttpClientPool,
    url: &str,
) -> Result<SharedPixelBuffer<Rgb8Pixel>> {
    log::info!(" 1. Free heap before: {}", unsafe {
        esp_idf_svc::sys::esp_get_minimum_free_heap_size()
    });
    match pool.get_bytes(url) {
        Ok(data) => {
            log::info!(" 2. Free heap before: {}", unsafe {
                esp_idf_svc::sys::esp_get_minimum_free_heap_size()
            });
            log::info!(
                "First 10 bytes of fetched image in hex: {:?}\nLast 10 bytes of fetched image in hex: {:?}\n Total length: {}",
                data.as_slice()[..10]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<String>>()
                    .join(" "),
                data.as_slice()[data.len() - 10..]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<String>>()
                    .join(" "),
                data.len()
            );
            let mut decoder = zune_jpeg::JpegDecoder::new(data.as_slice());
            log::info!(" 3. Free heap before: {}", unsafe {
                esp_idf_svc::sys::esp_get_minimum_free_heap_size()
            });
            decoder.decode_headers().unwrap();
            log::info!(" 4. Free heap before: {}", unsafe {
                esp_idf_svc::sys::esp_get_minimum_free_heap_size()
            });
            let image_info = decoder.info().unwrap();
            log::info!("Image info: {}x{}", image_info.width, image_info.height);
            let mut image = SharedPixelBuffer::<Rgb8Pixel>::new(
                image_info.width as u32,
                image_info.height as u32,
            );
            log::info!(" 5. Free heap before: {}", unsafe {
                esp_idf_svc::sys::esp_get_minimum_free_heap_size()
            });
            let _ = decoder.decode_into(image.make_mut_bytes());
            log::info!(" 6. Free heap before: {}", unsafe {
                esp_idf_svc::sys::esp_get_minimum_free_heap_size()
            });
            log::info!(
                "First 10 bytes of image in hex: {:?}",
                image.as_bytes()[..10]
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            Ok(image)
        }
        Err(e) => {
            log::error!("Failed to fetch image: {}", e);
            Err(e)
        }
    }
}
