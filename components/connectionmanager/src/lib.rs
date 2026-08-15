use jio_addressmanager::AddressManager;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct ConnectionManager {
    address_manager: AddressManager,
    active_connections: Arc<RwLock<HashSet<SocketAddr>>>,
    target_outbound: usize,
}

impl ConnectionManager {
    pub fn new(address_manager: AddressManager, target_outbound: usize) -> Self {
        Self {
            address_manager,
            active_connections: Arc::new(RwLock::new(HashSet::new())),
            target_outbound,
        }
    }

    pub fn is_connected(&self, addr: &SocketAddr) -> bool {
        self.active_connections.read().contains(addr)
    }

    pub fn register_connection(&self, addr: SocketAddr) {
        self.active_connections.write().insert(addr);
    }

    pub fn unregister_connection(&self, addr: &SocketAddr) {
        self.active_connections.write().remove(addr);
    }

    pub fn active_count(&self) -> usize {
        self.active_connections.read().len()
    }

    pub fn needs_more_outbound(&self) -> bool {
        self.active_count() < self.target_outbound
    }

    pub fn get_next_peer_candidate(&self) -> Option<SocketAddr> {
        if let Some(net_addr) = self.address_manager.get_address() {
            if !self.is_connected(&net_addr.addr) {
                return Some(net_addr.addr);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_connection_manager() {
        let addr_mgr = AddressManager::new();
        let conn_mgr = ConnectionManager::new(addr_mgr.clone(), 8);

        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
        let addr = SocketAddr::new(ip, 16111);

        addr_mgr.add_address(addr);
        assert!(conn_mgr.needs_more_outbound());
        assert_eq!(conn_mgr.get_next_peer_candidate(), Some(addr));

        conn_mgr.register_connection(addr);
        assert_eq!(conn_mgr.active_count(), 1);
        assert!(conn_mgr.is_connected(&addr));
        assert_eq!(conn_mgr.get_next_peer_candidate(), None);

        conn_mgr.unregister_connection(&addr);
        assert_eq!(conn_mgr.active_count(), 0);
        assert!(!conn_mgr.is_connected(&addr));
    }
}
