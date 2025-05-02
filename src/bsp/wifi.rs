use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi},
};

use log::info;

use anyhow::{anyhow, Result};

pub struct Wifi {
    wifi_driver: AsyncWifi<EspWifi<'static>>,
}

impl Wifi {
    pub async fn init(
        modem: Modem,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
        timer_service: EspTaskTimerService,
    ) -> Result<Self> {
        let wifi_driver = AsyncWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
            sys_loop,
            timer_service,
        )?;

        Ok(Self { wifi_driver })
    }

    pub async fn connect(&mut self, ssid: &str, password: &str) -> Result<()> {
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

        self.wifi_driver.wait_netif_up().await?;
        info!("WiFi interface is up");

        Ok(())
    }
}
