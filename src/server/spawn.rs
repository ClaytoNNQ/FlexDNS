use crate::{config::Config, cache::DnsCache, dns::DNSQueryHandler};
use tokio_util::sync::CancellationToken;
use std::sync::Arc;
use anyhow::Result;
use crate::server::{start_tcp_server, start_udp_server};

pub async fn spawn_servers(cfg: Arc<Config>, dns_cache: Arc<DnsCache>, tcp_token: &mut Option<CancellationToken>, udp_token: &mut Option<CancellationToken>, dns_token: &mut Option<CancellationToken>) -> Result<Arc<DNSQueryHandler>> {

    if let Some(token) = tcp_token.take() { token.cancel(); }
    if let Some(token) = udp_token.take() { token.cancel(); }
    if let Some(token) = dns_token.take() { token.cancel(); }

    let new_tcp_token = CancellationToken::new();
    let new_udp_token = CancellationToken::new();
    let new_dns_token = CancellationToken::new();

    *tcp_token = Some(new_tcp_token.clone());
    *udp_token = Some(new_udp_token.clone());
    *dns_token = Some(new_dns_token.clone());

    let handler = Arc::new(DNSQueryHandler::new(cfg.upstream.servers.clone(), cfg.upstream.retry_count, new_dns_token.clone()).await?);

    if cfg.listener.enable_tcp {
        let cfg_clone = cfg.clone();
        let cache_clone = dns_cache.clone();
        let handler_clone = handler.clone();
        let token_clone = new_tcp_token.clone();
        tokio::spawn(async move {
            let _ = start_tcp_server(cache_clone, &cfg_clone.listener.bind_addr, cfg_clone.listener.bind_port, token_clone, handler_clone).await;
        });
    }

    if cfg.listener.enable_udp {
        let cfg_clone = cfg.clone();
        let cache_clone = dns_cache.clone();
        let handler_clone = handler.clone();
        let token_clone = new_udp_token.clone();
        tokio::spawn(async move {
            let _ = start_udp_server(cache_clone, &cfg_clone.listener.bind_addr, cfg_clone.listener.bind_port, token_clone, handler_clone).await;
        });
    }

    Ok(handler)
}