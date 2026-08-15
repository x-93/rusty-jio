use crate::model::stores::statuses::StatusesStore;
use jio_consensus_core::blockhash::BlockHash;
use jio_consensus_core::blockstatus::BlockStatus;

#[derive(Clone)]
pub struct StatusesService {
    store: StatusesStore,
}

impl StatusesService {
    pub fn new(store: StatusesStore) -> Self {
        Self { store }
    }

    pub fn get(&self, hash: &BlockHash) -> Option<BlockStatus> {
        self.store.get(hash)
    }

    pub fn set(&self, hash: BlockHash, status: BlockStatus) {
        self.store.set(hash, status);
    }
}
