use esp_deck::{
    bsp::{time, wifi::Wifi},
    events::AppEvent,
    keyboard::Keyboard,
    ui::Window,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        prelude::*,
        task::{block_on, thread::ThreadSpawnConfiguration},
    },
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

#[toml_cfg::toml_config]
struct AppConfig {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
    #[default(5.5)]
    tz_offset: f32,
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Booting up...");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let (tx, rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();

    // This sets the configuration for the next threads that are spawned
    // I"m using it mainly to set the stack size to 4096
    ThreadSpawnConfiguration {
        stack_size: 4096,
        ..Default::default()
    }
    .set()
    .unwrap();
    let mut threads = Vec::new();
    threads.push(thread::spawn(move || {
        let mut wifi_driver = block_on(Wifi::init(
            peripherals.modem,
            sys_loop,
            nvs,
            timer_service,
            tx.clone(),
        ))
        .unwrap();

        block_on(wifi_driver.connect(APP_CONFIG.wifi_ssid, APP_CONFIG.wifi_password)).unwrap();

        // Set up NTP now that wifi is connected
        let _ = block_on(time::init(tx.clone())).unwrap();
        log::info!("NTP set up");

        // Keep the thread alive to preserve resources initialized in this block from being dropped
        loop {
            std::thread::sleep(std::time::Duration::from_secs(100));
        }
    }));

    threads.push(thread::spawn(move || {
        let _keyboard = Keyboard::new();
    }));
    // Set back to default to not influence other threads
    ThreadSpawnConfiguration::default().set().unwrap();

    let touch_i2c = esp_idf_svc::hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &esp_idf_svc::hal::i2c::config::Config::new().baudrate(400_000.Hz()),
    )?;

    let _ = Window::init(touch_i2c, rx, APP_CONFIG.tz_offset);

    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}
