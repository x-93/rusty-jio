use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetAddress {
    pub addr: SocketAddr,
    pub last_seen: u64,
    pub connection_failed_count: u32,
}

impl NetAddress {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            last_seen: jio_core::time::unix_now(),
            connection_failed_count: 0,
        }
    }
}

#[derive(Default, Clone)]
pub struct AddressStore {
    addresses: Arc<RwLock<HashSet<NetAddress>>>,
}

impl AddressStore {
    pub fn new() -> Self {
        Self {
            addresses: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn insert(&self, addr: NetAddress) {
        self.addresses.write().insert(addr);
    }

    pub fn remove(&self, addr: &SocketAddr) {
        self.addresses.write().retain(|a| a.addr != *addr);
    }

    pub fn get_all(&self) -> Vec<NetAddress> {
        self.addresses.read().iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.addresses.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.addresses.read().is_empty()
    }
}
