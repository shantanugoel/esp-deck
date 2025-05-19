use crate::events::AppEvent;
use anyhow::Result;
use embedded_svc::http::Headers;
use esp_idf_svc::{
    http::server::{EspHttpServer, Method},
    io::{Read, Write},
};
use serde::Deserialize;
use std::sync::mpsc::Sender;

pub fn register_all_http_handlers(
    server: &mut EspHttpServer,
    ui_tx: Sender<AppEvent>,
) -> anyhow::Result<()> {
    register_user_status_handler(server, ui_tx)?;
    Ok(())
}

const MAX_BODY_SIZE: usize = 1024;

fn read_body<R: Read>(
    reader: &mut R,
    max_size: usize,
    expected_size: Option<usize>,
) -> Result<Vec<u8>> {
    let mut body = Vec::with_capacity(expected_size.unwrap_or(1024));
    let mut total_read = 0;
    let mut buf = [0u8; 1024];

    if let Some(size) = expected_size {
        if size > max_size {
            return Err(anyhow::anyhow!("Request body too large"));
        }
        while total_read < size {
            let to_read = std::cmp::min(buf.len(), size - total_read);
            let n = reader
                .read(&mut buf[..to_read])
                .map_err(|e| anyhow::anyhow!("Read error: {:?}", e))?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&buf[..n]);
            total_read += n;
        }
    } else {
        // Unknown content length: read until EOF
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|e| anyhow::anyhow!("Read error: {:?}", e))?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&buf[..n]);
            total_read += n;
            if total_read > max_size {
                return Err(anyhow::anyhow!("Request body too large"));
            }
        }
    }
    Ok(body)
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct UserStatus {
    pub text: String,
    pub bgcolor: Option<[u8; 3]>,
}
fn register_user_status_handler(server: &mut EspHttpServer, ui_tx: Sender<AppEvent>) -> Result<()> {
    server.fn_handler("/user-status", Method::Post, move |mut request| {
        let content_len = request.content_len().map(|v| v as usize);
        let body = match read_body(&mut request, MAX_BODY_SIZE, content_len) {
            Ok(b) => b,
            Err(e) => {
                return request
                    .into_status_response(413)?
                    .write_all(format!("Request body error: {e}").as_bytes());
            }
        };

        let user_status = match serde_json::from_slice::<UserStatus>(&body) {
            Ok(val) => val,
            Err(_) => UserStatus {
                text: "".to_string(),
                bgcolor: None,
            },
        };
        if !user_status.text.is_empty() {
            ui_tx
                .send(AppEvent::UserStatusUpdate(user_status.clone()))
                .ok();
            request.into_ok_response()?.write_all(b"OK")
        } else {
            request
                .into_status_response(400)?
                .write_all(b"Missing 'text' field")
        }
    })?;
    Ok(())
}
