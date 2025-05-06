use esp_deck::{
    actor::Actor,
    bsp::{time, usb::Usb, wifi::Wifi},
    config::DeviceConfiguration,
    events::{AppEvent, WifiStatus},
    mapper::Mapper,
    ui::Window,
    usb_hid_client::UsbHidClient,
};
use esp_idf_svc::sys::{self as idf_sys, esp_vfs_littlefs_conf_t};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        prelude::*,
        task::{block_on, thread::ThreadSpawnConfiguration},
    },
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
};
use std::ffi::CString;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

const TZ_OFFSET: f32 = 5.5;
const VFS_BASE_PATH: &str = "/littlefs";
const CONFIG_PATH: &str = "/littlefs/device_config.json";
const PARTITION_LABEL: &str = "storage";

/// Mounts the LittleFS partition using the underlying C API.
fn init_vfs() -> anyhow::Result<()> {
    log::info!("Initializing VFS and mounting LittleFS via sys API...");

    let base_path = CString::new(VFS_BASE_PATH).unwrap();
    let partition_label = CString::new(PARTITION_LABEL).unwrap();

    // Rely entirely on Default, assuming it sets reasonable values or the C func handles it.
    let mut conf = esp_vfs_littlefs_conf_t {
        base_path: base_path.as_ptr(),
        partition_label: partition_label.as_ptr(),
        ..Default::default()
    };

    conf.set_format_if_mount_failed(true as u8);
    conf.set_dont_mount(false as u8);
    let ret = unsafe { idf_sys::esp_vfs_littlefs_register(&conf) };

    // Use full path for esp! macro
    esp_idf_svc::sys::esp!(ret)?;

    log::info!("LittleFS mounted successfully at {}", VFS_BASE_PATH);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Booting up...");

    // Attempt VFS init, but expect it to potentially fail silently for now
    if let Err(e) = init_vfs() {
        log::error!(
            "Failed to initialize VFS: {}. File operations will likely fail.",
            e
        );
    }

    // Load configuration - This will likely fail if VFS isn't mounted
    let config = DeviceConfiguration::load_or_create_default_config(CONFIG_PATH)?;

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let (tx, rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();
    let (actor_tx, actor_rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();
    let (usb_hid_tx, usb_hid_rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();

    ThreadSpawnConfiguration {
        stack_size: 4096,
        ..Default::default()
    }
    .set()
    .unwrap();
    let mut threads = Vec::new();

    let wifi_settings = config.settings.wifi.clone();
    let wifi_nvs = nvs;
    let wifi_sys_loop = sys_loop;
    let wifi_timer = timer_service.clone();
    let wifi_tx = tx.clone();
    let wifi_modem = peripherals.modem;

    threads.push(thread::spawn(move || {
        let mut wifi_driver = block_on(Wifi::init(
            wifi_settings,
            wifi_modem,
            wifi_sys_loop,
            wifi_nvs,
            wifi_timer,
            wifi_tx,
        ))
        .unwrap();

        match block_on(wifi_driver.connect()) {
            Ok(_) => {
                let _ = block_on(time::init(tx.clone())).unwrap();
                log::info!("NTP set up");
            }
            Err(e) => {
                log::error!("Wi-Fi connection failed: {}", e);
                tx.send(AppEvent::WifiUpdate(WifiStatus::Error(e.to_string())))
                    .unwrap();
            }
        }

        loop {
            std::thread::sleep(std::time::Duration::from_secs(100));
        }
    }));

    let actor_mappings = config.mappings.clone();
    let actor_mapper = Mapper::new(actor_mappings);
    let actor_usb_hid_tx = usb_hid_tx.clone();
    threads.push(thread::spawn(move || {
        let actor = Actor::new(actor_rx, actor_usb_hid_tx, actor_mapper);
        actor.run();
    }));

    threads.push(thread::spawn(move || {
        let _usb = Usb::new();
        UsbHidClient::run(usb_hid_rx).unwrap();
    }));
    ThreadSpawnConfiguration::default().set().unwrap();

    let touch_i2c = esp_idf_svc::hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &esp_idf_svc::hal::i2c::config::Config::new().baudrate(400_000.Hz()),
    )?;

    let _ = Window::init(touch_i2c, rx, actor_tx, TZ_OFFSET);

    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}
