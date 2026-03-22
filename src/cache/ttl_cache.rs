use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;

#[derive(Clone)]
struct CacheEntry {
    value: Arc<Vec<u8>>,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct DnsCache {
    inner: Arc<DashMap<Vec<u8>, CacheEntry>>,
    default_ttl: Duration,
}

impl DnsCache {
    pub fn new(default_ttl: Duration) -> Self {
        Self { inner: Arc::new(DashMap::new()), default_ttl }
    }

    pub fn insert(&self, packet: Vec<u8>, response: Vec<u8>, ttl: Option<Duration>) {
        let expire = Instant::now() + ttl.unwrap_or(self.default_ttl);
        self.inner.insert(packet, CacheEntry { value: Arc::new(response), expires_at: expire });
    }

    pub fn get(&self, packet: &[u8]) -> Option<Arc<Vec<u8>>> {
        if let Some(entry) = self.inner.get(packet) {
            return Some(entry.value.clone());
        }
        return None;
    }

    pub fn start_purge_task(self: Arc<Self>, purge_interval: Duration) {
        tokio::spawn(async move {
            let mut ticker = interval(purge_interval);
            loop {
                ticker.tick().await;
                let now = Instant::now();
                self.inner.retain(|_, entry| entry.expires_at > now);
            }
        });
    }
}