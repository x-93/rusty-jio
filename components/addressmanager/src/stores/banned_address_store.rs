use parking_lot::RwLock;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct BannedAddressStore {
    banned: Arc<RwLock<HashMap<IpAddr, u64>>>,
}

impl BannedAddressStore {
    pub fn new() -> Self {
        Self {
            banned: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn ban(&self, ip: IpAddr, until: u64) {
        self.banned.write().insert(ip, until);
    }

    pub fn unban(&self, ip: &IpAddr) {
        self.banned.write().remove(ip);
    }

    pub fn is_banned(&self, ip: &IpAddr) -> bool {
        let now = jio_core::time::unix_now();
        if let Some(&until) = self.banned.read().get(ip) {
            if now < until {
                return true;
            }
        }
        false
    }
}
