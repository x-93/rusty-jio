use super::utxo_entry::UtxoEntry;
use crate::tx::TransactionOutpoint;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type UtxoCollection = IndexMap<TransactionOutpoint, UtxoEntry>;

/// Represents an atomic state diff on the UTXO set: UTXOs added and UTXOs removed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtxoDiff {
    pub add: UtxoCollection,
    pub remove: UtxoCollection,
}

impl UtxoDiff {
    pub fn new(add: UtxoCollection, remove: UtxoCollection) -> Self {
        Self { add, remove }
    }

    /// Reverses the diff (adds become removes, removes become adds).
    pub fn reversed(&self) -> Self {
        Self {
            add: self.remove.clone(),
            remove: self.add.clone(),
        }
    }

    /// Applies another diff on top of this one to form a composite diff.
    pub fn with_diff(&mut self, other: &UtxoDiff) -> Result<(), &'static str> {
        for (outpoint, entry) in &other.remove {
            if self.add.shift_remove(outpoint).is_none() {
                self.remove.insert(*outpoint, entry.clone());
            }
        }
        for (outpoint, entry) in &other.add {
            if self.remove.shift_remove(outpoint).is_none() {
                self.add.insert(*outpoint, entry.clone());
            }
        }
        Ok(())
    }
}
