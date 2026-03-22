use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize)]
pub struct UpstreamConfig {
    pub retry_count: u8,
    pub servers: Vec<Upstream>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Upstream {
    pub priority: u16,
    #[serde(flatten)]
    pub transport: Transport,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Transport {
    #[serde(rename = "udp")]
    Udp { address: SocketAddr },
    #[serde(rename = "tcp")]
    Tcp { address: SocketAddr },
    #[serde(rename = "dot")]
    Dot { address: SocketAddr, server_name: String},
    #[serde(rename = "doh")]
    Doh { url: String, method: DohMethod},
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DohMethod {
    Get,
    Post,
}