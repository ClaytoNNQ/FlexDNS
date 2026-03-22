use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use std::net::SocketAddr;
use std::time::Duration;
use anyhow::Result;

use crate::dns::parse;

pub struct TcpClient {
    stream: Mutex<Option<TcpStream>>,
}

impl TcpClient {
    pub fn new() -> Self {
        Self { stream: Mutex::new(None) }
    }

    async fn connect(&self, addr: SocketAddr) -> Result<TcpStream> {
        Ok(tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(addr)).await??)
    }

    pub async fn send_query(&self, addr: SocketAddr, packet: &[u8]) -> Result<(Vec<u8>, Duration)> {
        let mut guard = self.stream.lock().await;

        if guard.is_none() {
            *guard = Some(self.connect(addr).await?);
        }

        let stream = guard.as_mut().unwrap();

        let len_bytes = (packet.len() as u16).to_be_bytes();
        stream.write_all(&len_bytes).await?;
        stream.write_all(packet).await?;

        let mut len_buf = [0u8; 2];
        stream.read_exact(&mut len_buf).await?;
        let resp_len = u16::from_be_bytes(len_buf) as usize;

        let mut resp = vec![0u8; resp_len];
        stream.read_exact(&mut resp).await?;


        let ttl = parse(&resp);
        Ok((resp, ttl))
    }
}