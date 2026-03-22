use std::net::{Ipv4Addr, SocketAddr};

use crate::config::upstream::{Transport, Upstream, UpstreamConfig};

pub fn append_dhcp_dns(cfg: &mut UpstreamConfig, dns_ips: &[Ipv4Addr]) {
    let mut priority = cfg.servers.iter().map(|s| s.priority).max().unwrap_or(0) + 1;

    for ip in dns_ips {
        let addr = SocketAddr::new((*ip).into(), 53);

        cfg.servers.push(Upstream {
            priority,
            transport: Transport::Udp { address: addr },
        });

        priority += 1;

        cfg.servers.push(Upstream {
            priority,
            transport: Transport::Tcp { address: addr },
        });

        priority += 1;
    }
}