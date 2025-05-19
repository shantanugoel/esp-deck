use crate::events::AppEvent;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use std::sync::mpsc::Sender;

/// Starts the HTTP server. Returns Some(EspHttpServer) if started, None if failed.
/// The register_handlers closure is called with a mutable reference to the server to register all handlers.
pub fn start_http_server<F>(
    ui_tx: Sender<AppEvent>,
    register_handlers: F,
) -> Option<EspHttpServer<'static>>
where
    F: Fn(&mut EspHttpServer<'static>, Sender<AppEvent>) -> anyhow::Result<()> + 'static,
{
    let mut server = match EspHttpServer::new(&Configuration::default()) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to start HTTP server: {}", e);
            return None;
        }
    };
    if let Err(e) = register_handlers(&mut server, ui_tx.clone()) {
        log::error!("Failed to register HTTP handlers: {}", e);
        return None;
    }
    Some(server)
}
