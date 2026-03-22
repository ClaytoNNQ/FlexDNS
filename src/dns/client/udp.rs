use tokio::net::UdpSocket;
use tokio::sync::oneshot;

use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use std::sync::Arc;
use std::time::{Duration};

use dashmap::DashMap;
use anyhow::{Result, anyhow};
use std::sync::atomic::{AtomicU16, Ordering};

struct PendingRequest {
    tx: oneshot::Sender<(Vec<u8>, Duration)>,
    addr: SocketAddr,
}

pub struct UdpClient {
    socket: Arc<UdpSocket>,
    pending: Arc<DashMap<u16, PendingRequest>>,
    timeout: Duration,
    id_counter: AtomicU16,
}

impl UdpClient {
    pub async fn new() -> Result<Self> {
        let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
        let socket = Arc::new(UdpSocket::bind(addr).await?);
        let pending = Arc::new(DashMap::new());
        let client = Self { socket: socket.clone(), pending: pending.clone(), timeout: Duration::from_secs(3), id_counter: AtomicU16::new(0) };
        tokio::spawn(Self::recv_loop(socket, pending));
        Ok(client)
    }

    pub async fn send_query(&self, addr: SocketAddr, packet: &[u8]) -> Result<(Vec<u8>, Duration)> {
        let id = self.id_counter.fetch_add(1, Ordering::Relaxed);
        let mut packet = packet.to_vec();
        packet[0..2].copy_from_slice(&id.to_be_bytes());


        let (tx, rx) = oneshot::channel();

        self.pending.insert(id, PendingRequest { tx, addr });

        if let Err(_) = self.socket.send_to(&packet, addr).await {
            self.pending.remove(&id);
            return Err(anyhow!(""));
        }

        match tokio::time::timeout(self.timeout, rx).await {
            Ok(Ok((resp, ttl))) => Ok((resp, ttl)),
            _ => {
                self.pending.remove(&id);
                Err(anyhow!(""))
            }
        }
    }

    async fn recv_loop(socket: Arc<UdpSocket>, pending: Arc<DashMap<u16, PendingRequest>>) {
        let mut buf = vec![0u8; 4096];

        loop {
            let (len, src) = match socket.recv_from(&mut buf).await {
                Ok(v) => v,
                Err(_) => continue,
            };
            if len < 2 { continue; }
            let id = u16::from_be_bytes([buf[0], buf[1]]);
            if let Some((_, req)) = pending.remove(&id) {
                if req.addr != src {
                    continue;
                }
                let ttl = crate::dns::parse(&buf[..len]);
                let resp = buf[..len].to_vec();
                let _ = req.tx.send((resp, ttl));
            }
        }
    }
}