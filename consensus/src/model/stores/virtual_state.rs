use jio_consensus_core::blockhash::BlockHash;
use jio_hashes::Hash;
use jio_utils::mem_size::MemSizeEstimator;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::mem::size_of;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualState {
    pub parents: Vec<BlockHash>,
    pub daa_score: u64,
    pub bits: u32,
    pub past_median_time: u64,
    pub blue_score: u64,
    pub selected_parent: BlockHash,
    pub utxo_commitment: Hash,
}

impl MemSizeEstimator for VirtualState {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.parents.len() * size_of::<BlockHash>()
    }
}

pub trait VirtualStateStoreReader {
    fn get(&self) -> Option<Arc<VirtualState>>;
}

#[derive(Default, Clone)]
pub struct VirtualStateStore {
    state: Arc<RwLock<Option<Arc<VirtualState>>>>,
}

impl VirtualStateStore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set(&self, state: Arc<VirtualState>) {
        *self.state.write() = Some(state);
    }

    pub fn get(&self) -> Option<Arc<VirtualState>> {
        self.state.read().clone()
    }
}

impl VirtualStateStoreReader for VirtualStateStore {
    fn get(&self) -> Option<Arc<VirtualState>> {
        self.state.read().clone()
    }
}
