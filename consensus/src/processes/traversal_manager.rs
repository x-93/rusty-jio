use crate::model::stores::relations::RelationsStore;
use jio_consensus_core::blockhash::BlockHash;

#[derive(Clone)]
pub struct TraversalManager {
    relations: RelationsStore,
}

impl TraversalManager {
    pub fn new(relations: RelationsStore) -> Self {
        Self { relations }
    }

    pub fn get_parents(&self, hash: &BlockHash) -> Option<Vec<BlockHash>> {
        self.relations.get_parents(hash)
    }
}
