use jio_consensus_core::tx::TransactionOutpoint;
use jio_consensus_core::utxo::{UtxoCollection, UtxoDiff, UtxoEntry, UtxoError, UtxoView};
use parking_lot::RwLock;
use std::sync::Arc;

pub trait UtxoSetStoreReader: UtxoView {
    fn get_collection(&self) -> UtxoCollection;
}

#[derive(Default, Clone)]
pub struct UtxoSetStore {
    utxos: Arc<RwLock<UtxoCollection>>,
}

impl UtxoSetStore {
    pub fn new() -> Self {
        Self {
            utxos: Arc::new(RwLock::new(UtxoCollection::new())),
        }
    }

    pub fn insert(&self, outpoint: TransactionOutpoint, entry: UtxoEntry) {
        self.utxos.write().insert(outpoint, entry);
    }

    pub fn remove(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.utxos.write().remove(outpoint)
    }

    pub fn apply_diff(&self, diff: &UtxoDiff) -> Result<(), UtxoError> {
        let mut utxos = self.utxos.write();
        diff.apply_to_collection(&mut utxos)
    }

    pub fn get_collection(&self) -> UtxoCollection {
        self.utxos.read().clone()
    }

    pub fn set_collection(&self, collection: UtxoCollection) {
        *self.utxos.write() = collection;
    }
}

impl UtxoView for UtxoSetStore {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.utxos.read().get(outpoint)
    }
}

impl UtxoSetStoreReader for UtxoSetStore {
    fn get_collection(&self) -> UtxoCollection {
        self.utxos.read().clone()
    }
}
