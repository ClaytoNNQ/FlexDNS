use reqwest::Client;
use anyhow::Result;
use base64::Engine;
use std::time::Duration;
use crate::{dns::parse, config::upstream::DohMethod};

pub struct DohClient {
    client: Client,
}

impl DohClient {
    pub fn new() -> Result<Self> {
        Ok(Self { client: Client::builder().timeout(Duration::from_secs(3)).build()? })
    }

    pub async fn send_query(&self, url: &str, packet: &[u8], method: DohMethod) -> Result<(Vec<u8>, Duration)> {
        println!("doh");
        let response = match method {
            DohMethod::Post => {
                self.client.post(url).header("Content-Type", "application/dns-message").header("Accept", "application/dns-message").body(packet.to_vec()).send().await?
            }
            DohMethod::Get => {
                let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(packet);
                let full_url = format!("{}?dns={}", url, encoded);
                self.client.get(full_url).header("Accept", "application/dns-message").send().await?
            }
        };

        let bytes = response.bytes().await?;
        let ttl = parse(&bytes);

        Ok((bytes.to_vec(), ttl))
    }
}