use crate::model::stores::relations::RelationsStore;
use jio_consensus_core::blockhash::BlockHash;

#[derive(Clone)]
pub struct RelationsManager {
    relations: RelationsStore,
}

impl RelationsManager {
    pub fn new(relations: RelationsStore) -> Self {
        Self { relations }
    }

    pub fn insert_relations(&self, hash: BlockHash, parents: Vec<BlockHash>) {
        self.relations.insert(hash, parents);
    }
}
