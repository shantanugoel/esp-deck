use esp_deck::{
    actor::Actor,
    bsp::{time, usb::Usb, wifi::Wifi},
    config::{Configurator, WifiSettings},
    events::{AppEvent, WifiStatus},
    mapper::Mapper,
    protocol::ProtocolManager,
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
    sync::mpsc::{self, Receiver, Sender, SyncSender},
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
    let mut config = Configurator::load_or_create_default_config(CONFIG_PATH)?;

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
    // Send a signal beforehand to ensure the wifi driver is initialized
    main_wifi_time_init_tx.send(wifi_settings).unwrap();

    threads.push(thread::spawn(move || {
        let mut wifi_driver = block_on(Wifi::init(
            wifi_modem,
            wifi_sys_loop,
            wifi_nvs,
            wifi_timer,
            peripheral_update_tx.clone(),
        ))
        .unwrap();

        loop {
            match main_wifi_time_init_rx.recv() {
                Ok(wifi_settings) => match block_on(wifi_driver.connect(wifi_settings)) {
                    Ok(_) => {
                        let _ = block_on(time::init(peripheral_update_tx.clone())).unwrap();
                        log::info!("NTP set up");
                    }
                    Err(e) => {
                        log::error!("Wi-Fi connection failed: {}", e);
                        peripheral_update_tx
                            .send(AppEvent::WifiUpdate(WifiStatus::Error(e.to_string())))
                            .unwrap();
                    }
                },
                _ => {
                    log::error!("Received spurious error signal from main_wifi_time_init_rx");
                }
            }
        }
    }));

    let actor_mappings = config.get_mappings().unwrap();
    let actor_mapper = Mapper::new(actor_mappings);
    let actor_usb_hid_tx = usb_hid_tx.clone();
    threads.push(thread::spawn(move || {
        let actor = Actor::new(actor_rx, actor_usb_hid_tx, actor_mapper);
        actor.run();
    }));

    let usb_updates_tx = ui_updates_tx.clone();
    let usb_message_tx = usb_message_tx.clone();
    threads.push(thread::spawn(move || {
        let _usb = Usb::new(usb_updates_tx.clone(), usb_message_tx.clone());
        UsbHidClient::run(usb_hid_rx).unwrap();
    }));
    ThreadSpawnConfiguration::default().set().unwrap();

    // Get the TZ offset here because we move the config into the ProtocolManager past this point
    let tz_offset = config.get_timezone_offset().unwrap_or(TZ_OFFSET);

    // Get the button names here because we move the config into the ProtocolManager past this point
    let button_names = config.get_button_names();

    threads.push(thread::spawn(move || {
        let protocol_manager =
            ProtocolManager::new(usb_message_rx, main_wifi_time_init_tx, &mut config);
        protocol_manager.run();
    }));

    let touch_i2c = esp_idf_svc::hal::i2c::I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &esp_idf_svc::hal::i2c::config::Config::new().baudrate(400_000.Hz()),
    )?;

    let _ = Window::init(touch_i2c, ui_updates_rx, actor_tx, tz_offset, button_names);

    for thread in threads {
        thread.join().unwrap();
    }

    Ok(())
}
