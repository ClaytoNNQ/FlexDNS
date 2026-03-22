use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use crate::cache::DnsCache;
use crate::dns::DNSQueryHandler;
use crate::server::parse_cache_key;

pub async fn start_tcp_server(cache: Arc<DnsCache>, bind_ip: &str, port: u16, cancel_token: CancellationToken, handler: Arc<DNSQueryHandler>) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", bind_ip, port)).await?;
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => break,

            accept_res = listener.accept() => {
                let (mut socket, _) = match accept_res {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let cache = Arc::clone(&cache);
                let handler = Arc::clone(&handler);

                tokio::spawn(async move {
                    loop {
                        let mut len_buf = [0u8; 2];
                        if socket.read_exact(&mut len_buf).await.is_err() { break; }
                        let msg_len = u16::from_be_bytes(len_buf) as usize;
                        if msg_len == 0 || msg_len > 65535 { break; }
                        let mut msg = vec![0u8; msg_len];
                        if socket.read_exact(&mut msg).await.is_err() { break; }
        
                        let query_id = [msg[0], msg[1]];
                        let key = parse_cache_key(&msg);

                        let mut response: Vec<u8> = if let Some(cached) = cache.get(&key) {
                            (*cached).clone()
                        } else {
                            let (resolved_bytes, ttl) =
                                match handler.dnsquery(&msg, true).await {
                                    Ok(res) => res,
                                    Err(_) => (Vec::new(), Duration::from_secs(0)),
                                };

                            cache.insert(key.to_vec(), resolved_bytes.clone(), Some(Duration::from_secs(300) + ttl * 10));
                            resolved_bytes
                        };
                        if response.len() >= 2 {
                            response[0] = query_id[0];
                            response[1] = query_id[1];
                        }

                        let mut full = Vec::with_capacity(2 + response.len());
                        full.extend_from_slice(&(response.len() as u16).to_be_bytes());
                        full.extend_from_slice(&response);

                        if socket.write_all(&full).await.is_err() { break; }
                    }
                });
            }
        }
    }
    Ok(())
}