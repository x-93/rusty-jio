//! Read-only view trait for looking up UTXOs.

use super::super::tx::{TransactionOutpoint, UtxoEntry};

pub trait UtxoView: Send + Sync {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry>;
}

impl UtxoView for super::utxo_collection::UtxoCollection {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.get(outpoint).cloned()
    }
}
