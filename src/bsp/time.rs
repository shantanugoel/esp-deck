use anyhow::Result;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};

pub async fn init() -> Result<EspSntp<'static>> {
    let sntp = EspSntp::new_default()?;

    while sntp.get_sync_status() != SyncStatus::Completed {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(sntp)
}
