use jio_consensus_core::blockhash::BlockHash;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait DaaStoreReader {
    fn get_daa_score(&self, hash: Hash) -> Option<u64>;
    fn get_bits(&self, hash: Hash) -> Option<u32>;
}

#[derive(Default, Clone)]
pub struct DaaStore {
    daa_scores: Arc<RwLock<HashMap<Hash, u64>>>,
    bits: Arc<RwLock<HashMap<Hash, u32>>>,
}

impl DaaStore {
    pub fn new() -> Self {
        Self {
            daa_scores: Arc::new(RwLock::new(HashMap::new())),
            bits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, daa_score: u64, bit: u32) {
        self.daa_scores.write().insert(hash, daa_score);
        self.bits.write().insert(hash, bit);
    }

    pub fn delete(&self, hash: Hash) {
        self.daa_scores.write().remove(&hash);
        self.bits.write().remove(&hash);
    }

    pub fn get_daa_score(&self, hash: &BlockHash) -> Option<u64> {
        <Self as DaaStoreReader>::get_daa_score(self, *hash)
    }

    pub fn get_bits(&self, hash: &BlockHash) -> Option<u32> {
        <Self as DaaStoreReader>::get_bits(self, *hash)
    }
}

impl DaaStoreReader for DaaStore {
    fn get_daa_score(&self, hash: Hash) -> Option<u64> {
        self.daa_scores.read().get(&hash).copied()
    }

    fn get_bits(&self, hash: Hash) -> Option<u32> {
        self.bits.read().get(&hash).copied()
    }
}
