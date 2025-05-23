use anyhow::Result;
use embedded_svc::http::client::{Client, Method};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use std::sync::mpsc::{channel, Sender};
use std::thread;

struct HttpClient {
    client: Client<EspHttpConnection>,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let config = Configuration {
            use_global_ca_store: true,
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            ..Default::default()
        };

        let connection = EspHttpConnection::new(&config)?;
        Ok(Self {
            client: Client::wrap(connection),
        })
    }

    pub fn get(&mut self, url: &str, headers: Option<&Vec<(String, String)>>) -> Result<Vec<u8>> {
        let headers_processed = match headers {
            Some(h) => h
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect::<Vec<(&str, &str)>>(),
            None => vec![("accept", "text/plain")],
        };
        let request = self.client.request(Method::Get, url, &headers_processed)?;
        let mut response = request.submit()?;

        let mut response_bytes = Vec::new();
        let mut buf = [0; 1024];

        loop {
            match response.read(&mut buf) {
                Ok(0) => break,
                Ok(bytes_read_count) => {
                    response_bytes.extend_from_slice(&buf[..bytes_read_count]);
                }
                Err(e) => return Err(anyhow::Error::from(e)),
            }
        }

        Ok(response_bytes)
    }
}

pub struct HttpRequest {
    pub url: String,
    pub response_tx: Sender<Result<Vec<u8>>>,
}

pub struct HttpClientPool {
    request_tx: Sender<HttpRequest>,
}

impl HttpClientPool {
    pub fn new() -> Self {
        let (request_tx, request_rx) = channel::<HttpRequest>();
        thread::spawn(move || {
            while let Ok(req) = request_rx.recv() {
                let mut client = match HttpClient::new() {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = req.response_tx.send(Err(e));
                        continue;
                    }
                };
                let result = client.get(&req.url, None);
                let _ = req.response_tx.send(result);
            }
        });
        Self { request_tx }
    }

    pub fn get(&self, url: &str) -> Result<String> {
        match self.get_bytes(url) {
            Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let (tx, rx) = channel();
        let req = HttpRequest {
            url: url.to_string(),
            response_tx: tx,
        };
        self.request_tx.send(req).unwrap();
        match rx.recv() {
            Ok(result) => match result {
                Ok(bytes) => Ok(bytes),
                Err(e) => Err(anyhow::Error::from(e)),
            },
            Err(e) => Err(anyhow::Error::from(e)),
        }
    }

    // Optionally, add async or callback-based API here
}

impl Default for HttpClientPool {
    fn default() -> Self {
        Self::new()
    }
}
