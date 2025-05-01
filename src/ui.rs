use anyhow::Result;
use esp_idf_svc::hal::i2c::I2cDriver;

use crate::bsp::slint_platform;

slint::include_modules!();
pub struct Window {
    window: MainWindow,
}

impl Window {
    pub fn init(touch_i2c: I2cDriver<'static>) -> Result<Self> {
        slint_platform::init(touch_i2c);
        let window = MainWindow::new()
            .map_err(|e| anyhow::anyhow!("Failed to create main window: {}", e))?;

        install_test_callback(&window);

        window
            .run()
            .map_err(|e| anyhow::anyhow!("Failed to run main window: {}", e))?;

        Ok(Self { window })
    }
}

fn install_test_callback(window: &MainWindow) {
    let _ = window.as_weak();
    window.on_update_fact(move || {
        log::info!("Test callback");
    });
}
