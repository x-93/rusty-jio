use jio_consensus_core::blockhash::BlockHash;
use parking_lot::RwLock;
use std::sync::Arc;

pub trait SelectedChainStoreReader {
    fn get_tip(&self) -> Option<BlockHash>;
    fn get_by_index(&self, index: usize) -> Option<BlockHash>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn get_chain(&self) -> Vec<BlockHash>;
}

#[derive(Default, Clone)]
pub struct SelectedChainStore {
    tip: Arc<RwLock<Option<BlockHash>>>,
    chain: Arc<RwLock<Vec<BlockHash>>>,
}

impl SelectedChainStore {
    pub fn new() -> Self {
        Self {
            tip: Arc::new(RwLock::new(None)),
            chain: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_tip(&self, hash: BlockHash) {
        *self.tip.write() = Some(hash);
    }

    pub fn push(&self, hash: BlockHash) {
        let mut chain = self.chain.write();
        chain.push(hash);
        *self.tip.write() = Some(hash);
    }

    pub fn pop(&self) -> Option<BlockHash> {
        let mut chain = self.chain.write();
        let popped = chain.pop();
        *self.tip.write() = chain.last().copied();
        popped
    }

    pub fn get_tip(&self) -> Option<BlockHash> {
        <Self as SelectedChainStoreReader>::get_tip(self)
    }

    pub fn get_by_index(&self, index: usize) -> Option<BlockHash> {
        <Self as SelectedChainStoreReader>::get_by_index(self, index)
    }

    pub fn len(&self) -> usize {
        <Self as SelectedChainStoreReader>::len(self)
    }

    pub fn is_empty(&self) -> bool {
        <Self as SelectedChainStoreReader>::is_empty(self)
    }

    pub fn get_chain(&self) -> Vec<BlockHash> {
        <Self as SelectedChainStoreReader>::get_chain(self)
    }
}

impl SelectedChainStoreReader for SelectedChainStore {
    fn get_tip(&self) -> Option<BlockHash> {
        *self.tip.read()
    }

    fn get_by_index(&self, index: usize) -> Option<BlockHash> {
        self.chain.read().get(index).copied()
    }

    fn len(&self) -> usize {
        self.chain.read().len()
    }

    fn is_empty(&self) -> bool {
        self.chain.read().is_empty()
    }

    fn get_chain(&self) -> Vec<BlockHash> {
        self.chain.read().clone()
    }
}
