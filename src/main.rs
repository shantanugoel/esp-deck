use esp_deck::http_client::HttpClientPool;
use esp_deck::http_handlers;
use esp_deck::{
    actor::Actor,
    bsp::{time, usb::Usb, wifi::Wifi},
    config::{Configurator, WifiSettings},
    events::{AppEvent, WifiStatus},
    http_server::start_http_server,
    mapper::Mapper,
    protocol::ProtocolManager,
    ui::window::Window,
    usb_hid_client::UsbHidClient,
};
use esp_idf_svc::hal::gpio::{Pin, PinDriver, Pull};
use esp_idf_svc::sys::{self as idf_sys, esp_vfs_littlefs_conf_t, gpio_set_level};
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

    let mut touch_i2c = esp_idf_svc::hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &esp_idf_svc::hal::i2c::config::Config::new().baudrate(400_000.Hz()),
    )?;

    // Reset touch screen before using it
    // DO NOT REMOVE THIS.
    let _ = touch_i2c.write(0x24, &[0x1], 1000);
    let mut exio_value = [0xC];
    let _ = touch_i2c.write(0x38, &exio_value, 1000);
    std::thread::sleep(std::time::Duration::from_millis(100));
    unsafe {
        gpio_set_level(peripherals.pins.gpio4.pin(), 0);
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
    exio_value[0] = 0xE;
    let _ = touch_i2c.write(0x38, &exio_value, 1000);
    std::thread::sleep(std::time::Duration::from_millis(200));

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

    // Fetch API key from config for the HTTP server thread
    // Clone it here as the original `config` will be moved later.
    let http_server_api_key = config.get_api_key();

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
        let mut http_server_handle = None;
        loop {
            match main_wifi_time_init_rx.recv() {
                Ok(wifi_settings) => match block_on(wifi_driver.connect(wifi_settings)) {
                    Ok(_) => {
                        // WiFi connected: start HTTP server
                        if let Err(e) = block_on(time::init(peripheral_update_tx.clone())) {
                            log::error!("NTP setup failed: {}", e);
                        } else {
                            log::info!("NTP setup complete");
                        }
                        if http_server_handle.is_none() {
                            // Use the cloned api_key for the closure
                            http_server_handle = start_http_server(
                                peripheral_update_tx.clone(),
                                http_server_api_key.clone(),
                                |server, ui_tx, current_api_key| {
                                    http_handlers::register_all_http_handlers(
                                        server,
                                        ui_tx,
                                        current_api_key,
                                    )
                                },
                            );
                            if http_server_handle.is_some() {
                                log::info!("HTTP server started");
                                let _ = peripheral_update_tx.send(AppEvent::HttpServerUpdate(
                                    "HTTP server started".to_string(),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        // WiFi failed/disconnected: drop HTTP server
                        if http_server_handle.is_some() {
                            http_server_handle = None;
                            let _ = peripheral_update_tx.send(AppEvent::HttpServerUpdate(
                                "HTTP server stopped".to_string(),
                            ));
                        }
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
            log::error!("Failed to get mappings from config");
            return Err(anyhow::anyhow!("Failed to get mappings from config"));
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

    // Get the TZ offset, widgets config, button names here because we move the config into
    // the ProtocolManager past this point
    let tz_offset = config.get_timezone_offset().unwrap_or(TZ_OFFSET);
    let widgets = config.get_widgets();
    let button_names = config.get_button_names();

    // Note: api_key for http server is already fetched and cloned above (http_server_api_key)
    // config object will be moved into ProtocolManager thread now.

    let actor_protocol_tx = actor_tx.clone();
    threads.push(thread::Builder::new().stack_size(8192).spawn(move || {
        let protocol_manager = ProtocolManager::new(
            usb_message_rx,
            main_wifi_time_init_tx,
            actor_protocol_tx,
            &config,
        );
        protocol_manager.run();
    })?);

    // Should move this to an ISR maybe?
    let mut button_pin = PinDriver::input(peripherals.pins.gpio6)?;
    let _ = button_pin.set_pull(Pull::Up);
    let mut toggle_busy = false;
    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(200));
        if !button_pin.is_high() {
            let user_status_update = if toggle_busy {
                toggle_busy = false;
                http_handlers::UserStatus {
                    text: "".to_string(),
                    bgcolor: Some([0, 0, 0]),
                }
            } else {
                toggle_busy = true;
                http_handlers::UserStatus {
                    text: "BUSY!\nHERE BE DRAGONS!".to_string(),
                    bgcolor: Some([255, 0, 0]),
                }
            };
            let _ = ui_updates_tx.send(AppEvent::UserStatusUpdate(user_status_update));
            // Hackiest debounce ever, lol
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }
    });

    // TODO: Get a signal from wifi to http pool to start serving requests
    let http_pool = Arc::new(HttpClientPool::new());
    let _ = Window::init(
        touch_i2c,
        ui_updates_rx,
        actor_tx,
        tz_offset,
        button_names,
        http_pool,
        widgets,
    );

    for thread in threads {
        if let Err(e) = thread.join() {
            log::error!("Thread panicked: {:?}", e);
        }
    }

    Ok(())
}
