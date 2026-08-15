use jio_consensus_core::block::Block;
use jio_consensus_core::blockhash::BlockHash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct OrphanBlocksPool {
    orphans: Arc<RwLock<HashMap<BlockHash, Block>>>,
}

impl OrphanBlocksPool {
    pub fn new() -> Self {
        Self {
            orphans: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: BlockHash, block: Block) {
        self.orphans.write().insert(hash, block);
    }

    pub fn remove(&self, hash: &BlockHash) -> Option<Block> {
        self.orphans.write().remove(hash)
    }

    pub fn contains(&self, hash: &BlockHash) -> bool {
        self.orphans.read().contains_key(hash)
    }

    pub fn len(&self) -> usize {
        self.orphans.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.orphans.read().is_empty()
    }
}
