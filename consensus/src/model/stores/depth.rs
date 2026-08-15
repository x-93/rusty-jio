use jio_consensus_core::blockhash::BlockHash;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait DepthStoreReader {
    fn get(&self, hash: Hash) -> Option<u64>;
}

#[derive(Default, Clone)]
pub struct DepthStore {
    depths: Arc<RwLock<HashMap<Hash, u64>>>,
}

impl DepthStore {
    pub fn new() -> Self {
        Self {
            depths: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, depth: u64) {
        self.depths.write().insert(hash, depth);
    }

    pub fn delete(&self, hash: Hash) {
        self.depths.write().remove(&hash);
    }

    pub fn get(&self, hash: &BlockHash) -> Option<u64> {
        <Self as DepthStoreReader>::get(self, *hash)
    }
}

impl DepthStoreReader for DepthStore {
    fn get(&self, hash: Hash) -> Option<u64> {
        self.depths.read().get(&hash).copied()
    }
}
