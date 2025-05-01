use esp_deck::bsp::wifi::Wifi;
use esp_idf_svc::{
    hal::{peripherals::Peripherals, task::block_on},
    eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, timer::EspTaskTimerService,
};

#[toml_cfg::toml_config]
struct AppConfig {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
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

    let mut wifi_driver = block_on(Wifi::init(peripherals, sys_loop, nvs, timer_service))?;

    block_on(wifi_driver.connect(APP_CONFIG.wifi_ssid, APP_CONFIG.wifi_password))?;

    Ok(())
}
