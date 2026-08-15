use crate::core::peer::{Peer, PeerKey};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Hub {
    peers: Arc<RwLock<HashMap<PeerKey, Arc<Peer>>>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, peer: Arc<Peer>) {
        self.peers.write().insert(peer.key, peer);
    }

    pub fn unregister(&self, key: &PeerKey) -> Option<Arc<Peer>> {
        self.peers.write().remove(key)
    }

    pub fn get(&self, key: &PeerKey) -> Option<Arc<Peer>> {
        self.peers.read().get(key).cloned()
    }

    pub fn active_peers(&self) -> Vec<Arc<Peer>> {
        self.peers.read().values().cloned().collect()
    }

    pub fn peer_count(&self) -> usize {
        self.peers.read().len()
    }

    pub async fn broadcast(&self, data: Vec<u8>) {
        let peers = self.active_peers();
        for peer in peers {
            let _ = peer.router.enqueue_outgoing(data.clone()).await;
        }
    }
}
