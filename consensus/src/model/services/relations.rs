use crate::model::stores::relations::RelationsStore;
use jio_consensus_core::blockhash::BlockHash;

#[derive(Clone)]
pub struct RelationsService {
    store: RelationsStore,
}

impl RelationsService {
    pub fn new(store: RelationsStore) -> Self {
        Self { store }
    }

    pub fn get_parents(&self, hash: &BlockHash) -> Option<Vec<BlockHash>> {
        self.store.get_parents(hash)
    }

    pub fn get_children(&self, hash: &BlockHash) -> Option<Vec<BlockHash>> {
        self.store.get_children(hash)
    }
}
