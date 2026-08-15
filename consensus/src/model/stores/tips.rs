use jio_consensus_core::blockhash::BlockHash;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

pub trait TipsStoreReader {
    fn get_tips(&self) -> Arc<Vec<BlockHash>>;
    fn is_tip(&self, hash: Hash) -> bool;
}

#[derive(Default, Clone)]
pub struct TipsStore {
    tips: Arc<RwLock<HashSet<BlockHash>>>,
}

impl TipsStore {
    pub fn new() -> Self {
        Self {
            tips: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn add(&self, hash: Hash) {
        self.tips.write().insert(hash);
    }

    pub fn remove(&self, hash: &BlockHash) -> bool {
        self.tips.write().remove(hash)
    }

    pub fn get_tips(&self) -> Vec<BlockHash> {
        self.tips.read().iter().copied().collect()
    }
}

impl TipsStoreReader for TipsStore {
    fn get_tips(&self) -> Arc<Vec<BlockHash>> {
        Arc::new(self.tips.read().iter().copied().collect())
    }

    fn is_tip(&self, hash: Hash) -> bool {
        self.tips.read().contains(&hash)
    }
}
