pub mod listener;
pub mod upstream;
pub mod load;

use listener::ListenerConfig;
use upstream::UpstreamConfig;

use anyhow::{Result, anyhow};

use crate::config::upstream::{DohMethod, Transport};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub listener: ListenerConfig,
    pub upstream: UpstreamConfig,
}

impl Config {
    pub fn validate(self) -> Result<Self> {
        if self.upstream.servers.is_empty() || self.listener.bind_port == 0 {
            return Err(anyhow!(""));
        }

        let mut seen = std::collections::HashSet::new();
        for srv in &self.upstream.servers {
            if !seen.insert(srv.priority) {
                return Err(anyhow!(""));
            }

             match &srv.transport {
                Transport::Dot { server_name, .. } => {
                    if server_name.is_empty() {
                        return Err(anyhow::anyhow!(""));
                    }
                }
                Transport::Doh { url, method } => {
                    if url.is_empty() {
                        return Err(anyhow::anyhow!(""));
                    }
                    match method {
                        DohMethod::Get | DohMethod::Post => {}
                    }
                }
                _ => {}
            }
        }

        Ok(self)
    }
}