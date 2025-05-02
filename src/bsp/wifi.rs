use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};

use log::info;

use crate::events::{AppEvent, WifiStatus};
use anyhow::{anyhow, Result};
use std::sync::mpsc::Sender;

pub struct Wifi {
    wifi_driver: AsyncWifi<EspWifi<'static>>,
    tx: Sender<AppEvent>,
}

impl Wifi {
    pub async fn init(
        modem: Modem,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
        timer_service: EspTaskTimerService,
        tx: Sender<AppEvent>,
    ) -> Result<Self> {
        let wifi_driver = AsyncWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
            sys_loop,
            timer_service,
        )?;
        tx.send(AppEvent::WifiUpdate(WifiStatus::Initializing))?;

        Ok(Self { wifi_driver, tx })
    }

    pub async fn connect(&mut self, ssid: &str, password: &str) -> Result<()> {
        self.tx.send(AppEvent::WifiUpdate(WifiStatus::Connecting))?;
        let wifi_config: Configuration = Configuration::Client(ClientConfiguration {
            ssid: ssid.try_into().map_err(|_| anyhow!("Invalid SSID"))?,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: password
                .try_into()
                .map_err(|_| anyhow!("Invalid password"))?,
            channel: None,
            ..Default::default()
        });

        self.wifi_driver.set_configuration(&wifi_config)?;
        self.wifi_driver.start().await?;
        info!("Waiting for WiFi connection...");

        self.wifi_driver.connect().await?;
        info!("Connected to WiFi");

        match self.wifi_driver.wait_netif_up().await {
            Ok(_) => {
                self.tx.send(AppEvent::WifiUpdate(WifiStatus::Connected(
                    self.wifi_driver.wifi().sta_netif().get_ip_info()?.ip,
                )))?;
            }
            Err(e) => {
                self.tx
                    .send(AppEvent::WifiUpdate(WifiStatus::Error(e.to_string())))?;
            }
        }
        info!("WiFi interface is up");

        loop {
            // Infinite loop to keep the thread alive
            // May add monitoring here later
            // This wastes thread stack, but it's ok. We may combine this thread
            // with other inits later to optimize
            std::thread::sleep(std::time::Duration::from_secs(100));
        }
    }
}
