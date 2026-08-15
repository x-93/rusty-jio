pub mod port_mapping_extender;
pub mod stores;

pub use port_mapping_extender::*;
pub use stores::*;

use std::net::{IpAddr, SocketAddr};

#[derive(Clone, Default)]
pub struct AddressManager {
    address_store: AddressStore,
    banned_store: BannedAddressStore,
}

impl AddressManager {
    pub fn new() -> Self {
        Self {
            address_store: AddressStore::new(),
            banned_store: BannedAddressStore::new(),
        }
    }

    pub fn add_address(&self, addr: SocketAddr) {
        if !self.banned_store.is_banned(&addr.ip()) {
            self.address_store.insert(NetAddress::new(addr));
        }
    }

    pub fn get_address(&self) -> Option<NetAddress> {
        let addresses = self.address_store.get_all();
        addresses
            .into_iter()
            .find(|a| !self.banned_store.is_banned(&a.addr.ip()))
    }

    pub fn ban(&self, ip: IpAddr, duration_ms: u64) {
        let until = jio_core::time::unix_now() + duration_ms;
        self.banned_store.ban(ip, until);
    }

    pub fn is_banned(&self, ip: &IpAddr) -> bool {
        self.banned_store.is_banned(ip)
    }

    pub fn len(&self) -> usize {
        self.address_store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.address_store.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_address_manager_lifecycle() {
        let mgr = AddressManager::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let addr = SocketAddr::new(ip, 16111);

        assert_eq!(mgr.len(), 0);
        mgr.add_address(addr);
        assert_eq!(mgr.len(), 1);

        let candidate = mgr.get_address().expect("address found");
        assert_eq!(candidate.addr, addr);

        // Test banning
        assert!(!mgr.is_banned(&ip));
        mgr.ban(ip, 60_000);
        assert!(mgr.is_banned(&ip));
        assert!(mgr.get_address().is_none());
    }
}
