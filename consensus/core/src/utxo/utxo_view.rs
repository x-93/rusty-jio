use super::{UtxoCollection, UtxoEntry};
use crate::tx::TransactionOutpoint;

/// Trait providing read-only access to UTXO entries.
pub trait UtxoView {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry>;
}

impl UtxoView for UtxoCollection {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.get(outpoint).cloned()
    }
}
