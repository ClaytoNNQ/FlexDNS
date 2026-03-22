use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ListenerConfig {
    pub bind_addr: String,
    pub bind_port: u16,
    pub fallback_dhbc: bool,
    pub enable_udp: bool,
    pub enable_tcp: bool,
    pub enable_llmnr: bool,
    pub enable_mdns: bool,
}