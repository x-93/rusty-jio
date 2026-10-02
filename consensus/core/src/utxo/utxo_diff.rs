//! Atomic composition of UTXO set changes (additions and removals).

use super::utxo_collection::UtxoCollection;
use super::utxo_error::UtxoDiffError;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct UtxoDiff {
    pub to_add: UtxoCollection,
    pub to_remove: UtxoCollection,
}

impl UtxoDiff {
    pub fn new(to_add: UtxoCollection, to_remove: UtxoCollection) -> Self {
        Self { to_add, to_remove }
    }

    /// Composes this diff with a subsequent diff.
    pub fn with_diff(&self, other: &UtxoDiff) -> Result<UtxoDiff, UtxoDiffError> {
        let mut to_add = self.to_add.clone();
        let mut to_remove = self.to_remove.clone();

        for (outpoint, entry) in &other.to_remove {
            if to_add.remove(outpoint).is_none() {
                if to_remove.contains_key(outpoint) {
                    return Err(UtxoDiffError::DuplicateRemove(*outpoint));
                }
                to_remove.insert(*outpoint, entry.clone());
            }
        }

        for (outpoint, entry) in &other.to_add {
            if to_remove.remove(outpoint).is_none() {
                if to_add.contains_key(outpoint) {
                    return Err(UtxoDiffError::DuplicateAdd(*outpoint));
                }
                to_add.insert(*outpoint, entry.clone());
            }
        }

        Ok(UtxoDiff { to_add, to_remove })
    }

    /// Returns the inverted diff, swapping additions and removals.
    pub fn reversed(&self) -> Self {
        Self {
            to_add: self.to_remove.clone(),
            to_remove: self.to_add.clone(),
        }
    }

    /// Composes another diff into this diff in place.
    pub fn with_diff_in_place(&mut self, other: &UtxoDiff) -> Result<(), UtxoDiffError> {
        let composed = self.with_diff(other)?;
        *self = composed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{ScriptPublicKey, TransactionId, TransactionOutpoint, UtxoEntry};

    #[test]
    fn test_utxo_diff_composition_and_reversal() {
        let op1 = TransactionOutpoint::new(TransactionId::from_bytes([1; 32]), 0);
        let entry1 = UtxoEntry::new(100, ScriptPublicKey::default(), 1, false);

        let op2 = TransactionOutpoint::new(TransactionId::from_bytes([2; 32]), 0);
        let entry2 = UtxoEntry::new(200, ScriptPublicKey::default(), 2, false);

        let mut diff1 = UtxoDiff::default();
        diff1.to_add.insert(op1, entry1.clone());

        let mut diff2 = UtxoDiff::default();
        diff2.to_remove.insert(op1, entry1.clone());
        diff2.to_add.insert(op2, entry2.clone());

        let composed = diff1.with_diff(&diff2).unwrap();
        assert!(!composed.to_add.contains_key(&op1));
        assert!(!composed.to_remove.contains_key(&op1));
        assert!(composed.to_add.contains_key(&op2));

        let reversed = composed.reversed();
        assert!(reversed.to_remove.contains_key(&op2));
        assert!(!reversed.to_add.contains_key(&op2));
    }
}
