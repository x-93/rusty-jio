use crate::tx::TransactionOutpoint;
use crate::utxo::UtxoEntry;
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::{Hash, HasherBase, MuHashElementHash, MuHashFinalizeHash};
use serde::{Deserialize, Serialize};

/// Represents an incremental, commutative Multi-set Hash (MuHash) over UTXO entries.
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MuHash {
    pub state: [u64; 4],
}

impl Default for MuHash {
    fn default() -> Self {
        Self::new()
    }
}

impl MuHash {
    pub const fn new() -> Self {
        Self {
            state: [1, 0, 0, 0], // Multiplicative identity
        }
    }

    /// Adds a UTXO entry to the multi-set hash.
    pub fn add_utxo(&mut self, outpoint: &TransactionOutpoint, entry: &UtxoEntry) {
        let element_hash = Self::hash_utxo_element(outpoint, entry);
        let words = element_hash.to_le_u64();
        // Additive/multiplicative accumulation
        for (i, word) in words.iter().enumerate() {
            self.state[i] = self.state[i].wrapping_add(*word);
        }
    }

    /// Removes a UTXO entry from the multi-set hash.
    pub fn remove_utxo(&mut self, outpoint: &TransactionOutpoint, entry: &UtxoEntry) {
        let element_hash = Self::hash_utxo_element(outpoint, entry);
        let words = element_hash.to_le_u64();
        for (i, word) in words.iter().enumerate() {
            self.state[i] = self.state[i].wrapping_sub(*word);
        }
    }

    /// Computes the final 32-byte UTXO commitment hash.
    pub fn finalize(&self) -> Hash {
        let mut hasher = MuHashFinalizeHash::new();
        for word in &self.state {
            hasher.update(word.to_le_bytes());
        }
        hasher.finalize()
    }

    fn hash_utxo_element(outpoint: &TransactionOutpoint, entry: &UtxoEntry) -> Hash {
        let mut hasher = MuHashElementHash::new();
        hasher.update(outpoint.transaction_id.as_bytes());
        hasher.update(outpoint.index.to_le_bytes());
        hasher.update(entry.amount.to_le_bytes());
        hasher.update(entry.script_public_key.version.to_le_bytes());
        hasher.update((entry.script_public_key.script.len() as u64).to_le_bytes());
        hasher.update(&entry.script_public_key.script);
        hasher.update(entry.block_daa_score.to_le_bytes());
        hasher.update([entry.is_coinbase as u8]);
        hasher.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::ScriptPublicKey;

    #[test]
    fn test_muhash_commutativity() {
        let op1 = TransactionOutpoint::new(Hash::from([1u8; 32]), 0);
        let op2 = TransactionOutpoint::new(Hash::from([2u8; 32]), 1);

        let e1 = UtxoEntry::new(100, ScriptPublicKey::new(0, vec![0x51]), 10, false);
        let e2 = UtxoEntry::new(200, ScriptPublicKey::new(0, vec![0x51]), 20, false);

        // Add 1 then 2
        let mut muhash_a = MuHash::new();
        muhash_a.add_utxo(&op1, &e1);
        muhash_a.add_utxo(&op2, &e2);

        // Add 2 then 1 (commutativity check)
        let mut muhash_b = MuHash::new();
        muhash_b.add_utxo(&op2, &e2);
        muhash_b.add_utxo(&op1, &e1);

        assert_eq!(muhash_a.finalize(), muhash_b.finalize());

        // Remove 1
        muhash_a.remove_utxo(&op1, &e1);
        let mut muhash_only_2 = MuHash::new();
        muhash_only_2.add_utxo(&op2, &e2);

        assert_eq!(muhash_a.finalize(), muhash_only_2.finalize());
    }
}
