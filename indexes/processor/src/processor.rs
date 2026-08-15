use jio_consensus_core::block::Block;
use jio_consensus_core::utxo::UtxoDiff;
use jio_indexes_core::IndexedUtxos;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct IndexProcessor {
    indexed_utxos: Arc<RwLock<IndexedUtxos>>,
}

impl IndexProcessor {
    pub fn new() -> Self {
        Self {
            indexed_utxos: Arc::new(RwLock::new(IndexedUtxos::new())),
        }
    }

    pub fn process_utxo_diff(&self, diff: &UtxoDiff) {
        let mut index = self.indexed_utxos.write();
        for (outpoint, entry) in &diff.to_remove {
            index.remove(&entry.script_public_key, outpoint);
        }
        for (outpoint, entry) in &diff.to_add {
            index.insert(entry.script_public_key.clone(), *outpoint, entry.clone());
        }
    }

    pub fn process_block(&self, _block: &Block) {
        // Index block transactions if needed
    }

    pub fn indexed_utxos(&self) -> Arc<RwLock<IndexedUtxos>> {
        self.indexed_utxos.clone()
    }
}
