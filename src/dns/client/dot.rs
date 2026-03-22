use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::rustls::pki_types::ServerName;
use webpki_roots::TLS_SERVER_ROOTS;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;

use crate::dns::parse;

pub struct DotClient {
    tls_config: Arc<ClientConfig>,
    stream: Mutex<Option<TlsStream<TcpStream>>>,
}

impl DotClient {
    pub fn new() -> Result<Self> {
        let mut root_store = RootCertStore::empty();
        root_store.extend(TLS_SERVER_ROOTS.iter().cloned());
        let config = ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();
        Ok(Self {tls_config: Arc::new(config), stream: Mutex::new(None)})
    }

    async fn connect(&self, addr: SocketAddr, server_name: &str) -> Result<TlsStream<TcpStream>> {
        let tcp = tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(addr)).await??;
        let connector = TlsConnector::from(self.tls_config.clone());
        let domain = ServerName::try_from(server_name.to_string())?;
        let tls_stream = tokio::time::timeout(Duration::from_secs(3), connector.connect(domain, tcp)).await??;
        Ok(tls_stream)
    }

    pub async fn send_query(&self, addr: SocketAddr, server_name: &str, packet: &[u8]) -> Result<(Vec<u8>, Duration)> {

        let mut guard = self.stream.lock().await;

        if guard.is_none() {
            *guard = Some(self.connect(addr, server_name).await?);
        }

        let stream = guard.as_mut().unwrap();

        let resp = timeout(
            Duration::from_secs(5),
            async {
                let len_bytes = (packet.len() as u16).to_be_bytes();
                stream.write_all(&len_bytes).await?;
                stream.write_all(packet).await?;

                let mut len_buf = [0u8; 2];
                stream.read_exact(&mut len_buf).await?;

                let resp_len = u16::from_be_bytes(len_buf) as usize;

                let mut resp = vec![0u8; resp_len];
                stream.read_exact(&mut resp).await?;

                Ok::<Vec<u8>, anyhow::Error>(resp)
            }
        ).await??;

        let ttl = parse(&resp);
        Ok((resp, ttl))
    }
}