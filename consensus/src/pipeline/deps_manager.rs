use jio_consensus_core::blockhash::BlockHash;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct BlockTaskDependencyManager {
    pending: Arc<Mutex<HashMap<BlockHash, HashSet<BlockHash>>>>,
}

impl BlockTaskDependencyManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_dependent(&self, child: BlockHash, parent: BlockHash) {
        self.pending
            .lock()
            .entry(child)
            .or_default()
            .insert(parent);
    }

    pub fn satisfy_dependency(&self, parent: &BlockHash) -> Vec<BlockHash> {
        let mut pending = self.pending.lock();
        let mut ready = Vec::new();

        pending.retain(|child, parents| {
            parents.remove(parent);
            if parents.is_empty() {
                ready.push(*child);
                false
            } else {
                true
            }
        });

        ready
    }
}
