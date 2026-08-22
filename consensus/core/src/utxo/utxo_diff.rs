use super::{UtxoAlgebraError, UtxoCollection, UtxoCollectionExtensions};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Represents a mutation (diff) applied to the UTXO set: additions and removals.
#[derive(
    Clone, Default, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct UtxoDiff {
    pub to_add: UtxoCollection,
    pub to_remove: UtxoCollection,
}

impl UtxoDiff {
    pub fn new(to_add: UtxoCollection, to_remove: UtxoCollection) -> Self {
        Self { to_add, to_remove }
    }

    /// Returns a new UtxoDiff with reversed polarity (swap add and remove).
    pub fn reversed(&self) -> Self {
        Self {
            to_add: self.to_remove.clone(),
            to_remove: self.to_add.clone(),
        }
    }

    /// Consumes and returns a reversed UtxoDiff.
    pub fn with_reversed(self) -> Self {
        Self {
            to_add: self.to_remove,
            to_remove: self.to_add,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.to_add.is_empty() && self.to_remove.is_empty()
    }

    pub fn has_intersection(&self, other: &Self) -> bool {
        self.to_add.has_intersection(&other.to_add)
            || self.to_remove.has_intersection(&other.to_remove)
    }

    /// Composes another diff on top of this diff.
    pub fn diff_from(&self, other: &Self) -> Result<Self, UtxoAlgebraError> {
        let mut to_add = self.to_add.clone();
        let mut to_remove = self.to_remove.clone();

        for (outpoint, entry) in &other.to_add {
            if to_remove.remove(outpoint).is_none() {
                to_add.insert(*outpoint, entry.clone());
            }
        }

        for (outpoint, entry) in &other.to_remove {
            if to_add.remove(outpoint).is_none() {
                to_remove.insert(*outpoint, entry.clone());
            }
        }

        Ok(Self { to_add, to_remove })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::{ScriptPublicKey, TransactionId, TransactionOutpoint};
    use crate::utxo::UtxoEntry;

    #[test]
    fn test_utxo_diff_algebra() {
        let op1 = TransactionOutpoint::new(TransactionId::from([1u8; 32]), 0);
        let op2 = TransactionOutpoint::new(TransactionId::from([2u8; 32]), 1);

        let entry1 = UtxoEntry::new(100, ScriptPublicKey::new(0, vec![0x51]), 10, false);
        let entry2 = UtxoEntry::new(200, ScriptPublicKey::new(0, vec![0x51]), 20, false);

        let mut col_add = UtxoCollection::new();
        col_add.insert(op1, entry1.clone());

        let mut col_rem = UtxoCollection::new();
        col_rem.insert(op2, entry2.clone());

        let diff = UtxoDiff::new(col_add, col_rem);
        let reversed = diff.reversed();

        assert_eq!(reversed.to_add.len(), 1);
        assert_eq!(reversed.to_remove.len(), 1);
        assert!(reversed.to_add.contains_key(&op2));
        assert!(reversed.to_remove.contains_key(&op1));
    }
}
