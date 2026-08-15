use crate::core::router::Router;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub type PeerKey = SocketAddr;

pub struct Peer {
    pub key: PeerKey,
    pub router: Arc<Router>,
    pub is_outbound: bool,
    pub last_ping: AtomicU64,
}

impl Peer {
    pub fn new(key: PeerKey, router: Arc<Router>, is_outbound: bool) -> Self {
        Self {
            key,
            router,
            is_outbound,
            last_ping: AtomicU64::new(jio_core::time::unix_now()),
        }
    }

    pub fn set_last_ping(&self, timestamp: u64) {
        self.last_ping.store(timestamp, Ordering::Relaxed);
    }

    pub fn get_last_ping(&self) -> u64 {
        self.last_ping.load(Ordering::Relaxed)
    }
}
