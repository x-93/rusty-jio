use crate::tx::{TransactionOutpoint, UtxoEntry};
use std::collections::HashMap;

pub type UtxoCollection = HashMap<TransactionOutpoint, UtxoEntry>;

pub trait UtxoCollectionExtensions {
    fn contains_outpoint(&self, outpoint: &TransactionOutpoint) -> bool;
}

impl UtxoCollectionExtensions for UtxoCollection {
    fn contains_outpoint(&self, outpoint: &TransactionOutpoint) -> bool {
        self.contains_key(outpoint)
    }
}
