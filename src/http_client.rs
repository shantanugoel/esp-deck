use anyhow::Result;
use embedded_svc::http::client::{Client, Method};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};

pub struct HttpClient {
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

    pub fn get(&mut self, url: &str, headers: Option<&Vec<(String, String)>>) -> Result<String> {
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

        Ok(String::from_utf8_lossy(&response_bytes).into_owned())
    }
}
