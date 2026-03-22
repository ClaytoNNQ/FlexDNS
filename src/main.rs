mod config;
mod core;
mod cache;
mod server;
mod dhbc;
mod dns;

use cache::DnsCache;
use crate::config::load::load_config;
use std::{path::PathBuf, sync::Arc, time::{Duration, Instant}};
use tokio::sync::watch;
use anyhow::Result;

use crate::core::start_config_watcher;
use server::spawn::spawn_servers;

use nlink::netlink::{Connection, Route, RtnetlinkGroup, NetworkEvent};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = "/etc/flexdns/resolv.conf";

    let mut current_config = Arc::new(load_config(config_path)?);
    println!("{:?}", current_config);
    let mut dns_cache = Arc::new(DnsCache::new(Duration::from_secs(10)));
    dns_cache.clone().start_purge_task(Duration::from_secs(5));

    let (reload_tx, mut reload_rx) = watch::channel(());
    let _watcher = start_config_watcher(PathBuf::from(config_path), reload_tx)?;

    let mut tcp_token: Option<tokio_util::sync::CancellationToken> = None;
    let mut udp_token: Option<tokio_util::sync::CancellationToken> = None;
    let mut dns_token: Option<tokio_util::sync::CancellationToken> = None;

    let _handler = spawn_servers(current_config.clone(), dns_cache.clone(), &mut tcp_token, &mut udp_token, &mut dns_token).await?;

    let mut conn = Connection::<Route>::new()?;
    conn.subscribe(&[RtnetlinkGroup::Ipv4Addr])?;
    let mut net_events = conn.events();

    let mut last_reload = Instant::now() - Duration::from_secs(1);
    let debounce_duration = Duration::from_millis(500);
    
    loop {
        tokio::select! {
            _ = reload_rx.changed() => {
                if last_reload.elapsed() >= debounce_duration {
                    println!("Config değişikliği algılandı, DNS servisi resetleniyor");
                    
                    current_config = Arc::new(load_config(config_path)?);
                    println!("{:?}", current_config);

                    dns_cache = Arc::new(DnsCache::new(Duration::from_secs(5)));
                    dns_cache.clone().start_purge_task(Duration::from_secs(5));

                    let _handler = spawn_servers(current_config.clone(), dns_cache.clone(), &mut tcp_token, &mut udp_token, &mut dns_token).await?;

                    last_reload = Instant::now();
                }
            }

            Some(event) = net_events.next() => {
                if let NetworkEvent::NewAddress(addr) = event? && let Some(ip) = addr.address() && ip.is_ipv4() && !ip.is_loopback() && last_reload.elapsed() >= debounce_duration {
                    println!("Yeni IPv4 algılandı: {:?}", ip);
                    
                    current_config = Arc::new(load_config(config_path)?);
                    println!("{:?}", current_config);

                    dns_cache = Arc::new(DnsCache::new(Duration::from_secs(5)));
                    dns_cache.clone().start_purge_task(Duration::from_secs(5));

                    let _handler = spawn_servers(current_config.clone(), dns_cache.clone(), &mut tcp_token, &mut udp_token, &mut dns_token).await?;

                    last_reload = Instant::now();
                }
            }
        }
    }
}