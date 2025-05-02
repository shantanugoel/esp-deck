use anyhow::Result;
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use std::sync::mpsc::Sender;

use crate::events::{AppEvent, TimeStatus};

pub async fn init(tx: Sender<AppEvent>) -> Result<EspSntp<'static>> {
    tx.send(AppEvent::TimeUpdate(TimeStatus::Initializing))?;
    let sntp = EspSntp::new_default()?;

    while sntp.get_sync_status() != SyncStatus::Completed {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    tx.send(AppEvent::TimeUpdate(TimeStatus::Synced))?;

    Ok(sntp)
}
