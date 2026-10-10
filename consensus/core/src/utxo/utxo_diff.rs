use super::utxo_collection::UtxoCollection;
use super::utxo_error::UtxoAlgebraError;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct UtxoDiff {
    pub add: UtxoCollection,
    pub remove: UtxoCollection,
}

impl UtxoDiff {
    pub fn new(add: UtxoCollection, remove: UtxoCollection) -> Self {
        Self { add, remove }
    }

    pub fn with_diff(&mut self, other: &Self) -> Result<(), UtxoAlgebraError> {
        for (outpoint, entry) in &other.remove {
            if self.add.remove(outpoint).is_none() {
                self.remove.insert(*outpoint, entry.clone());
            }
        }
        for (outpoint, entry) in &other.add {
            if self.remove.remove(outpoint).is_none() {
                self.add.insert(*outpoint, entry.clone());
            }
        }
        Ok(())
    }

    pub fn diff_with(&self, other: &Self) -> Result<Self, UtxoAlgebraError> {
        let mut cloned = self.clone();
        cloned.with_diff(other)?;
        Ok(cloned)
    }

    pub fn with_reversed(&self) -> Self {
        Self {
            add: self.remove.clone(),
            remove: self.add.clone(),
        }
    }
}
