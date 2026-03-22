use anyhow::Result;
use config::{Config as RawConfig, File, FileFormat};
use std::fs;
use std::io::Write;

use crate::{
    config::Config,
    dhbc::{append_dhcp_dns, get_dhbc_info},
};

pub fn load_config(path: &str) -> Result<Config> {
    let settings = RawConfig::builder()
        .add_source(File::new(path, FileFormat::Json))
        .build();

    match settings {
        Ok(settings) => match settings.try_deserialize::<Config>() {
            Ok(cfg) => process_config(cfg),
            Err(e) => Err(e.into()),
        },
        Err(_) => write_default(path),
    }
}

fn process_config(mut cfg: Config) -> Result<Config> {
    if let Some(devices) = get_dhbc_info() && cfg.listener.fallback_dhbc {
        for dhcp_dns in devices {
            append_dhcp_dns(&mut cfg.upstream, &dhcp_dns.dns_servers);
        }
    }

    cfg.upstream.servers.sort_by_key(|u| u.priority);
    cfg.validate()
}

fn write_default(path: &str) -> Result<Config> {
    let default_string = r#"{
  "listener": {
    "bind_addr": "127.0.0.10",
    "bind_port": 53,
    "fallback_dhbc" : true,
    "enable_udp": true,
    "enable_tcp": true,
    "enable_llmnr": false,
    "enable_mdns": false
  },
  "upstream": {
    "retry_count": 2,
    "servers": [
      { "priority": 1, "type": "udp", "address": "1.1.1.1:53" },
      { "priority": 2, "type": "tcp", "address": "8.8.8.8:53" },
      { "priority": 3, "type": "dot", "address": "9.9.9.9:853", "server_name": "dns.quad9.net" },
      { "priority": 4, "type": "doh", "url": "https://cloudflare-dns.com/dns-query", "method": "get" },
      { "priority": 5, "type": "doh", "url": "https://dns.google/dns-query", "method": "post" }
    ]
  }
}"#;

    let tmp_path = format!("{}.tmp", path);

    let mut file = fs::File::create(&tmp_path)?;
    file.write_all(default_string.as_bytes())?;
    file.sync_all()?;
    fs::rename(&tmp_path, path)?;

    let settings = RawConfig::builder()
        .add_source(File::new(path, FileFormat::Json))
        .build()?;

    let cfg: Config = settings.try_deserialize()?;
    process_config(cfg)
}