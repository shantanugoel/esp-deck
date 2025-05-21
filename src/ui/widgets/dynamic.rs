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

static WIDGET_ITEMS: [WidgetItemData; 2] = [
    WidgetItemData {
        title: "Hello",
        kind: Kind::Text("Hello"),
    },
    WidgetItemData {
        title: "Hello2",
        kind: Kind::Image("https://shantanugoel.com/img/avatar.jpg"),
    },
];

pub fn start_dynamic_service(window: Weak<MainWindow>, http_pool: Arc<HttpClientPool>) {
    // Wait till wifi is connected
    std::thread::sleep(std::time::Duration::from_millis(10000));
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
    match pool.get_bytes(url) {
        Ok(mut data) => {
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
            let mut image_info: esp_jpeg_image_output_t = esp_jpeg_image_output_t::default();
            let mut image_cfg: esp_jpeg_image_cfg_t = esp_jpeg_image_cfg_t::default();
            image_cfg.indata = data.as_mut_ptr();
            image_cfg.indata_size = data.len() as u32;
            image_cfg.out_format = esp_jpeg_image_format_t_JPEG_IMAGE_FORMAT_RGB888;
            let image_info_result = unsafe {
                esp_jpeg_get_image_info(&mut image_cfg, &mut image_info);
            };
            log::info!(
                "Image info: {:?}, Result: {:?}",
                image_info,
                image_info_result
            );
            let mut image = SharedPixelBuffer::<Rgb8Pixel>::new(
                image_info.width as u32,
                image_info.height as u32,
            );
            image_cfg.outbuf = image.make_mut_bytes().as_mut_ptr();
            image_cfg.outbuf_size = image_info.output_len as u32;
            let mut image_output_info: esp_jpeg_image_output_t = esp_jpeg_image_output_t::default();
            unsafe {
                esp_jpeg_decode(&mut image_cfg, &mut image_output_info);
            }
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
