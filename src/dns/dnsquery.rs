use anyhow::Result;
use tokio_util::sync::CancellationToken;

use crate::{dns::{doh::DohClient, dot::DotClient, tcp::TcpClient, udp::UdpClient}, config::upstream::{Transport, Upstream}};

use std::sync::Arc;
use std::time::Duration;
pub struct DNSQueryHandler {
    pub upstreams: Vec<Upstream>,
    pub udp: Arc<UdpClient>,
    pub tcp: Arc<TcpClient>,
    pub dot: Arc<DotClient>,
    pub doh: Arc<DohClient>,
    pub retry_count: u8,
    pub _cancel: CancellationToken,
}

impl DNSQueryHandler {
    pub async fn new(upstreams: Vec<Upstream>, retry_count: u8, _cancel: CancellationToken) -> Result<Self> {
        let udp = Arc::new(UdpClient::new().await?);
        let tcp = Arc::new(TcpClient::new());
        let dot = Arc::new(DotClient::new()?);
        let doh = Arc::new(DohClient::new()?);

        Ok(Self { upstreams, udp, tcp, dot, doh, retry_count, _cancel })
    }

    pub async fn dnsquery(&self, packet: &[u8], is_tcp: bool) -> Result<(Vec<u8>, Duration)> {
        let mut last_resp: Option<(Vec<u8>, Duration)> = None;
        for _ in 0..self.retry_count {
            for upstream in &self.upstreams {
                let allowed = match is_tcp {
                    false => matches!(upstream.transport, Transport::Udp{..} | Transport::Tcp{..} | Transport::Dot{..} | Transport::Doh{..}),
                    true => matches!(upstream.transport, Transport::Tcp{..} | Transport::Dot{..} | Transport::Doh{..}),
                };

                if !allowed {
                    continue;
                }

                let result: Result<(Vec<u8>, Duration)> = match &upstream.transport {
                    Transport::Udp { address } => self.udp.send_query(*address, packet).await,
                    Transport::Tcp { address } => self.tcp.send_query(*address, packet).await,
                    Transport::Dot { address, server_name } => self.dot.send_query(*address, server_name, packet).await,
                    Transport::Doh { url, method } => self.doh.send_query(url, packet, method.clone()).await,
                };
                if let Ok((resp, dur)) = result {
                    if resp.len() >= 12 {
                        let rcode = resp[3] & 0x0F;
                        let ancount = u16::from_be_bytes([resp[6], resp[7]]);

                        if rcode == 2 || rcode == 3 {
                            last_resp = Some((resp, dur));
                            continue;
                        }
                        if rcode == 0 && ancount > 0 {
                            return Ok((resp, dur));
                        }
                        last_resp = Some((resp, dur));
                    }
                }
                
            }
        }

        if let Some(resp) = last_resp {
            return Ok(resp);
        }

        let mut resp = packet.to_vec();
        if resp.len() >= 12 {
            resp[2] |= 0x80;
            resp[3] = (resp[3] & 0xF0) | 2;
            resp[6] = 0; resp[7] = 0;
            resp[8] = 0; resp[9] = 0;
            resp[10] = 0; resp[11] = 0;
        }

        Ok((resp, Duration::ZERO))
    }
}