// utxo/utxo_diff.rs
//
// Represents the difference in UTXO state between two DAG points (additions vs. removals).

use super::utxo_collection::UtxoCollection;
use crate::tx::{TransactionOutpoint, UtxoEntry};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct UtxoDiff {
    pub to_add: UtxoCollection,
    pub to_remove: UtxoCollection,
}

impl UtxoDiff {
    #[inline]
    pub fn new(to_add: UtxoCollection, to_remove: UtxoCollection) -> Self {
        Self { to_add, to_remove }
    }

    #[inline]
    pub fn add_entry(&mut self, outpoint: TransactionOutpoint, entry: UtxoEntry) {
        self.to_add.insert(outpoint, entry);
    }

    #[inline]
    pub fn remove_entry(&mut self, outpoint: TransactionOutpoint, entry: UtxoEntry) {
        self.to_remove.insert(outpoint, entry);
    }
}
