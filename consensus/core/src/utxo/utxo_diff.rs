use crate::utxo::utxo_collection::UtxoCollection;
use crate::utxo::utxo_error::UtxoError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct UtxoDiff {
    pub to_add: UtxoCollection,
    pub to_remove: UtxoCollection,
}

impl UtxoDiff {
    pub fn new(to_add: UtxoCollection, to_remove: UtxoCollection) -> Self {
        Self { to_add, to_remove }
    }

    pub fn reverse(&self) -> Self {
        Self {
            to_add: self.to_remove.clone(),
            to_remove: self.to_add.clone(),
        }
    }

    pub fn apply_to_collection(&self, collection: &mut UtxoCollection) -> Result<(), UtxoError> {
        for (outpoint, _) in &self.to_remove {
            if collection.remove(outpoint).is_none() {
                return Err(UtxoError::NotFound);
            }
        }
        for (outpoint, entry) in &self.to_add {
            if collection.insert(*outpoint, entry.clone()).is_some() {
                return Err(UtxoError::Duplicate);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::TransactionOutpoint;
    use crate::utxo::utxo_collection::UtxoEntry;
    use jio_hashes::Hash;
    use jio_txscript::ScriptPublicKey;
    use std::collections::HashMap;

    #[test]
    fn test_utxo_diff_application_and_reversal() {
        let mut collection = UtxoCollection::new();

        let op1 = TransactionOutpoint::new(Hash::from_le_u64([1, 0, 0, 0]), 0);
        let op2 = TransactionOutpoint::new(Hash::from_le_u64([2, 0, 0, 0]), 1);
        let op3 = TransactionOutpoint::new(Hash::from_le_u64([3, 0, 0, 0]), 0);

        let entry1 = UtxoEntry::new(1000, ScriptPublicKey::new(0, vec![1]), 10, false);
        let entry2 = UtxoEntry::new(2000, ScriptPublicKey::new(0, vec![2]), 20, false);
        let entry3 = UtxoEntry::new(3000, ScriptPublicKey::new(0, vec![3]), 30, false);

        // Initial collection has op1
        collection.insert(op1, entry1.clone());

        // Diff: remove op1, add op2 and op3
        let mut to_add = HashMap::new();
        to_add.insert(op2, entry2.clone());
        to_add.insert(op3, entry3.clone());

        let mut to_remove = HashMap::new();
        to_remove.insert(op1, entry1.clone());

        let diff = UtxoDiff::new(to_add, to_remove);
        diff.apply_to_collection(&mut collection).unwrap();

        assert_eq!(collection.len(), 2);
        assert!(!collection.contains_key(&op1));
        assert_eq!(collection.get(&op2), Some(&entry2));
        assert_eq!(collection.get(&op3), Some(&entry3));

        // Reversal diff should restore original collection
        let reverse_diff = diff.reverse();
        reverse_diff.apply_to_collection(&mut collection).unwrap();

        assert_eq!(collection.len(), 1);
        assert_eq!(collection.get(&op1), Some(&entry1));
        assert!(!collection.contains_key(&op2));
        assert!(!collection.contains_key(&op3));
    }

    #[test]
    fn test_utxo_diff_not_found_error() {
        let mut collection = UtxoCollection::new();
        let op = TransactionOutpoint::new(Hash::from_le_u64([99, 0, 0, 0]), 0);
        let entry = UtxoEntry::new(500, ScriptPublicKey::default(), 1, false);

        let mut to_remove = HashMap::new();
        to_remove.insert(op, entry);

        let diff = UtxoDiff::new(HashMap::new(), to_remove);
        let err = diff.apply_to_collection(&mut collection).unwrap_err();
        assert_eq!(err, UtxoError::NotFound);
    }

    #[test]
    fn test_utxo_diff_duplicate_error() {
        let mut collection = UtxoCollection::new();
        let op = TransactionOutpoint::new(Hash::from_le_u64([99, 0, 0, 0]), 0);
        let entry = UtxoEntry::new(500, ScriptPublicKey::default(), 1, false);

        collection.insert(op, entry.clone());

        let mut to_add = HashMap::new();
        to_add.insert(op, entry);

        let diff = UtxoDiff::new(to_add, HashMap::new());
        let err = diff.apply_to_collection(&mut collection).unwrap_err();
        assert_eq!(err, UtxoError::Duplicate);
    }
}

