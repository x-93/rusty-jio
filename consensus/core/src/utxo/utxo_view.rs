use crate::tx::TransactionOutpoint;
use crate::utxo::utxo_collection::UtxoEntry;

pub trait UtxoView {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry>;
    fn has(&self, outpoint: &TransactionOutpoint) -> bool {
        self.get(outpoint).is_some()
    }
}

impl<T: std::ops::Deref<Target = crate::utxo::utxo_collection::UtxoCollection>> UtxoView for T {
    fn get(&self, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        self.deref().get(outpoint).cloned()
    }
}
