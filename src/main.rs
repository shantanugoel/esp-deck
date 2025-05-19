use esp_deck::http_client::HttpClientPool;
use esp_deck::{
    actor::Actor,
    bsp::{time, usb::Usb, wifi::Wifi},
    config::{Configurator, WifiSettings},
    events::{AppEvent, WifiStatus},
    mapper::Mapper,
    protocol::ProtocolManager,
    ui::window::Window,
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
    sync::mpsc::{self, Receiver, Sender, SyncSender},
    sync::Arc,
    thread,
};

const TZ_OFFSET: f32 = 5.5;
const VFS_BASE_PATH: &str = "/littlefs";
const CONFIG_PATH: &str = "/littlefs/device_config.json";
const PARTITION_LABEL: &str = "storage";

/// Mounts the LittleFS partition using the underlying C API.
fn init_vfs() -> anyhow::Result<()> {
    log::info!("Initializing VFS and mounting LittleFS via sys API...");

    let base_path = match CString::new(VFS_BASE_PATH) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create CString for base_path: {}", e);
            return Err(anyhow::anyhow!("Invalid base_path for VFS"));
        }
    };
    let partition_label = match CString::new(PARTITION_LABEL) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to create CString for partition_label: {}", e);
            return Err(anyhow::anyhow!("Invalid partition_label for VFS"));
        }
    };

    // Rely entirely on Default, assuming it sets reasonable values or the C func handles it.
    let mut conf = esp_vfs_littlefs_conf_t {
        base_path: base_path.as_ptr(),
        partition_label: partition_label.as_ptr(),
        ..Default::default()
    };

    // This is important to set for the very first time device boots, otherwise the VFS will not be mounted
    conf.set_format_if_mount_failed(true as u8);
    conf.set_dont_mount(false as u8);
    let ret = unsafe { idf_sys::esp_vfs_littlefs_register(&conf) };

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
    let config = Configurator::load_or_create_default_config(CONFIG_PATH)?;

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // UI Updates Channel is used to send events to the UI thread. Mostly for logging.
    let (ui_updates_tx, ui_updates_rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();
    // Actor would take action on events typically from the UI thread. (e.g. when a button is pressed, or a new config is received)
    // and it sends events to the underlying USB module
    let (actor_tx, actor_rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();
    let (usb_hid_tx, usb_hid_rx): (Sender<AppEvent>, Receiver<AppEvent>) = mpsc::channel();
    let (usb_message_tx, usb_message_rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
    let (main_wifi_time_init_tx, main_wifi_time_init_rx): (
        SyncSender<Option<WifiSettings>>,
        Receiver<Option<WifiSettings>>,
    ) = mpsc::sync_channel(1);

    ThreadSpawnConfiguration {
        stack_size: 4096,
        ..Default::default()
    }
    .set()
    .unwrap();
    let mut threads = Vec::new();

    let wifi_settings = config.get_wifi_settings();
    let wifi_nvs = nvs;
    let wifi_sys_loop = sys_loop;
    let wifi_timer = timer_service.clone();
    let peripheral_update_tx = ui_updates_tx.clone();
    let wifi_modem = peripherals.modem;
    if let Err(e) = main_wifi_time_init_tx.send(wifi_settings) {
        log::error!("Failed to send wifi settings: {}", e);
    }

    threads.push(thread::spawn(move || {
        let mut wifi_driver = match block_on(Wifi::init(
            wifi_modem,
            wifi_sys_loop,
            wifi_nvs,
            wifi_timer,
            peripheral_update_tx.clone(),
        )) {
            Ok(driver) => driver,
            Err(e) => {
                log::error!("Failed to initialize WiFi: {}", e);
                return;
            }
        };
        loop {
            match main_wifi_time_init_rx.recv() {
                Ok(wifi_settings) => match block_on(wifi_driver.connect(wifi_settings)) {
                    Ok(_) => {
                        if let Err(e) = block_on(time::init(peripheral_update_tx.clone())) {
                            log::error!("NTP setup failed: {}", e);
                        } else {
                            log::info!("NTP set up");
                        }
                    }
                    Err(e) => {
                        log::error!("Wi-Fi connection failed: {}", e);
                        if let Err(e2) = peripheral_update_tx
                            .send(AppEvent::WifiUpdate(WifiStatus::Error(e.to_string())))
                        {
                            log::error!("Failed to send WiFi error update: {}", e2);
                        }
                    }
                },
                Err(_) => {
                    log::error!("Received spurious error signal from main_wifi_time_init_rx");
                }
            }
        }
    }));

    let actor_mappings = match config.get_mappings() {
        Some(m) => m,
        None => {
            log::error!("Failed to get mappings");
            return Err(anyhow::anyhow!("Failed to get mappings"));
        }
    };
    let actor_mapper = Mapper::new(actor_mappings);
    let actor_usb_hid_tx = usb_hid_tx.clone();
    threads.push(thread::spawn(move || {
        let mut actor = Actor::new(actor_rx, actor_usb_hid_tx, actor_mapper);
        actor.run();
    }));

    let usb_updates_tx = ui_updates_tx.clone();
    let usb_message_tx = usb_message_tx.clone();
    threads.push(thread::spawn(move || {
        let _usb = Usb::new(usb_updates_tx.clone(), usb_message_tx.clone());
        if let Err(e) = UsbHidClient::run(usb_hid_rx) {
            log::error!("UsbHidClient::run failed: {}", e);
        }
    }));
    if let Err(e) = ThreadSpawnConfiguration::default().set() {
        log::error!("Failed to set thread spawn configuration: {}", e);
    }

    // Get the TZ offset here because we move the config into the ProtocolManager past this point
    let tz_offset = config.get_timezone_offset().unwrap_or(TZ_OFFSET);

    // Get the button names here because we move the config into the ProtocolManager past this point
    let button_names = config.get_button_names();

    let actor_protocol_tx = actor_tx.clone();
    threads.push(thread::spawn(move || {
        let protocol_manager = ProtocolManager::new(
            usb_message_rx,
            main_wifi_time_init_tx,
            actor_protocol_tx,
            &config,
        );
        protocol_manager.run();
    }));

    let touch_i2c = esp_idf_svc::hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &esp_idf_svc::hal::i2c::config::Config::new().baudrate(400_000.Hz()),
    )?;

    let http_pool = Arc::new(HttpClientPool::new());
    let _ = Window::init(
        touch_i2c,
        ui_updates_rx,
        actor_tx,
        tz_offset,
        button_names,
        http_pool,
    );

    for thread in threads {
        if let Err(e) = thread.join() {
            log::error!("Thread panicked: {:?}", e);
        }
    }

    Ok(())
}
