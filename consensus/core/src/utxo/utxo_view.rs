use crate::tx::{TransactionOutpoint, UtxoEntry};

pub trait UtxoView {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry>;
}

impl UtxoView for super::utxo_collection::UtxoCollection {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.get(outpoint).cloned()
    }
}
