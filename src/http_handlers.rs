use crate::events::AppEvent;
use anyhow::Result;
use embedded_svc::http::Headers;
use esp_idf_svc::{
    http::server::{EspHttpServer, Method},
    io::Write,
};
use std::sync::mpsc::Sender;

pub fn register_all_http_handlers(
    server: &mut EspHttpServer,
    ui_tx: Sender<AppEvent>,
) -> anyhow::Result<()> {
    register_user_status_handler(server, ui_tx)?;
    Ok(())
}

const MAX_BODY_SIZE: usize = 1024;

fn register_user_status_handler(server: &mut EspHttpServer, ui_tx: Sender<AppEvent>) -> Result<()> {
    server.fn_handler("/user-status", Method::Post, move |mut request| {
        let body_size = request.content_len().unwrap_or(0) as usize;

        if body_size > MAX_BODY_SIZE {
            return request
                .into_status_response(413)?
                .write_all(b"Request body too large");
        }

        let mut body = Vec::with_capacity(body_size);
        let mut total_read = 0;
        let mut buf = [0u8; 1024];
        while total_read < body_size {
            let to_read = std::cmp::min(buf.len(), body_size - total_read);
            let n = request.read(&mut buf[..to_read])?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&buf[..n]);
            total_read += n;
        }

        // Fallback for unknown content length (read until EOF)
        if body_size == 0 {
            loop {
                let n = request.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                body.extend_from_slice(&buf[..n]);
            }
        }

        let text = match serde_json::from_slice::<serde_json::Value>(&body) {
            Ok(val) => val
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            Err(_) => "".to_string(),
        };
        if !text.is_empty() {
            ui_tx.send(AppEvent::UserStatusUpdate(text.clone())).ok();
            request.into_ok_response()?.write_all(b"OK")
        } else {
            request
                .into_status_response(400)?
                .write_all(b"Missing 'text' field")
        }
    })?;
    Ok(())
}
