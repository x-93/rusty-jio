use super::UtxoEntry;
use crate::tx::TransactionOutpoint;
use std::collections::HashMap;

/// A collection of UTXO entries indexed by transaction outpoint.
pub type UtxoCollection = HashMap<TransactionOutpoint, UtxoEntry>;

pub trait UtxoCollectionExtensions {
    fn total_amount(&self) -> u64;
    fn has_intersection(&self, other: &UtxoCollection) -> bool;
}

impl UtxoCollectionExtensions for UtxoCollection {
    fn total_amount(&self) -> u64 {
        self.values().map(|entry| entry.amount).sum()
    }

    fn has_intersection(&self, other: &UtxoCollection) -> bool {
        if self.len() <= other.len() {
            self.keys().any(|k| other.contains_key(k))
        } else {
            other.keys().any(|k| self.contains_key(k))
        }
    }
}
