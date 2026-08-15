use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::blockstatus::BlockStatus;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait StatusesStoreReader {
    fn get(&self, hash: Hash) -> Option<BlockStatus>;
    fn has(&self, hash: Hash) -> bool;
}

#[derive(Default, Clone)]
pub struct StatusesStore {
    statuses: Arc<RwLock<HashMap<Hash, BlockStatus>>>,
}

impl StatusesStore {
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set(&self, hash: Hash, status: BlockStatus) {
        self.statuses.write().insert(hash, status);
    }

    pub fn delete(&self, hash: Hash) {
        self.statuses.write().remove(&hash);
    }
}

impl StatusesStoreReader for StatusesStore {
    fn get(&self, hash: Hash) -> Option<BlockStatus> {
        self.statuses.read().get(&hash).copied()
    }

    fn has(&self, hash: Hash) -> bool {
        self.statuses.read().contains_key(&hash)
    }
}

// Ergonomic helper methods accepting both &BlockHash and BlockHash
impl StatusesStore {
    pub fn get(&self, hash: &BlockHash) -> Option<BlockStatus> {
        <Self as StatusesStoreReader>::get(self, *hash)
    }

    pub fn has(&self, hash: &BlockHash) -> bool {
        <Self as StatusesStoreReader>::has(self, *hash)
    }
}
