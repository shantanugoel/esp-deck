use slint::{SharedString, Timer, Weak};

use crate::ui::window::MainWindow;
use std::time::Duration;

pub fn start_server_widget_service(window: Weak<MainWindow>) {
    let timer = Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(10),
        move || {
            let window_clone = window.clone().upgrade().unwrap();
            let free_heap = unsafe { esp_idf_svc::sys::esp_get_free_heap_size() };
            let min_free_heap = unsafe { esp_idf_svc::sys::esp_get_minimum_free_heap_size() };
            window_clone.set_server_widget_1_title(SharedString::from("Device Status".to_string()));
            window_clone.set_server_widget_1_value(SharedString::from(format!(
                "Free Heap: {}kB\nMin Free Heap: {}kB",
                free_heap / 1024,
                min_free_heap / 1024
            )));
        },
    );
    Box::leak(Box::new(timer));
}
