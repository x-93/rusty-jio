use jio_consensus_core::acceptance_data::AcceptanceData;
use jio_consensus_core::blockhash::BlockHash;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait AcceptanceDataStoreReader {
    fn get(&self, hash: Hash) -> Option<Arc<AcceptanceData>>;
    fn has(&self, hash: Hash) -> bool;
}

#[derive(Default, Clone)]
pub struct AcceptanceDataStore {
    data: Arc<RwLock<HashMap<Hash, Arc<AcceptanceData>>>>,
}

impl AcceptanceDataStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, acceptance_data: Arc<AcceptanceData>) {
        self.data.write().insert(hash, acceptance_data);
    }

    pub fn delete(&self, hash: Hash) {
        self.data.write().remove(&hash);
    }

    pub fn get(&self, hash: &BlockHash) -> Option<Arc<AcceptanceData>> {
        <Self as AcceptanceDataStoreReader>::get(self, *hash)
    }

    pub fn has(&self, hash: &BlockHash) -> bool {
        <Self as AcceptanceDataStoreReader>::has(self, *hash)
    }
}

impl AcceptanceDataStoreReader for AcceptanceDataStore {
    fn get(&self, hash: Hash) -> Option<Arc<AcceptanceData>> {
        self.data.read().get(&hash).cloned()
    }

    fn has(&self, hash: Hash) -> bool {
        self.data.read().contains_key(&hash)
    }
}
