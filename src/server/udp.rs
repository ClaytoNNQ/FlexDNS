use crate::{cache::DnsCache, server::parse_cache_key};
use crate::dns::DNSQueryHandler;
use std::sync::Arc;
use tokio::{net::UdpSocket, time::{timeout, Duration}};
use tokio_util::sync::CancellationToken;

pub async fn start_udp_server(cache: Arc<DnsCache>, bind_ip: &str, port: u16, cancel_token: CancellationToken, handler: Arc<DNSQueryHandler>) -> tokio::io::Result<()> {
    let socket = Arc::new(UdpSocket::bind(format!("{}:{}", bind_ip, port)).await?);
    let mut buf = vec![0u8; 512];
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => break,
            recv_res = timeout(Duration::from_secs(2), socket.recv_from(&mut buf)) => {
                let (len, addr) = match recv_res {
                    Ok(Ok(res)) => res,
                    _ => continue,
                };
                let query_bytes = buf[..len].to_vec();

                let cache = Arc::clone(&cache);
                let handler = Arc::clone(&handler);
                let socket = Arc::clone(&socket);

                tokio::spawn(async move {
                    let query_id = [query_bytes[0], query_bytes[1]];
                    let key = parse_cache_key(&query_bytes);
                    let mut response: Vec<u8> =
                        if let Some(cached) = cache.get(&key) {
                            (*cached).clone()
                        } else {
                            let (resolved_bytes, ttl) =
                                match handler.dnsquery(&query_bytes, false).await {
                                    Ok(res) => res,
                                    Err(_) => (Vec::new(), Duration::from_secs(0)),
                                };
                            cache.insert(key.to_vec(), resolved_bytes.clone(), Some(Duration::from_secs(100) + ttl * 5));

                            resolved_bytes
                        };
                    if response.len() >= 2 {
                        response[0] = query_id[0];
                        response[1] = query_id[1];
                    }
                    let _ = socket.send_to(&response, addr).await;
                });
            }
        }
    }
    Ok(())
}