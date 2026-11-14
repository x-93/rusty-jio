use crate::tx::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::{Hash, HasherBase, TransactionSigningHash, TransactionSigningHashECDSA};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct SigHashType(u8);

impl SigHashType {
    pub const ALL: Self = Self(0b00000001);
    pub const NONE: Self = Self(0b00000010);
    pub const SINGLE: Self = Self(0b00000100);
    pub const ANYONECANPAY: Self = Self(0b10000000);

    #[inline(always)]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    #[inline(always)]
    pub const fn to_bits(&self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn is_all(&self) -> bool {
        self.0 & Self::ALL.0 != 0
    }

    #[inline(always)]
    pub fn is_none(&self) -> bool {
        self.0 & Self::NONE.0 != 0
    }

    #[inline(always)]
    pub fn is_single(&self) -> bool {
        self.0 & Self::SINGLE.0 != 0
    }

    #[inline(always)]
    pub fn is_anyone_can_pay(&self) -> bool {
        self.0 & Self::ANYONECANPAY.0 != 0
    }
}

impl Default for SigHashType {
    fn default() -> Self {
        Self::ALL
    }
}

/// Computes the Schnorr signature hash for a given transaction input.
pub fn calc_schnorr_signature_hash(
    tx: &Transaction,
    input_idx: usize,
    hash_type: SigHashType,
) -> Hash {
    let mut hasher = TransactionSigningHash::new();
    hasher.update(tx.version.to_le_bytes());
    hasher.update([hash_type.to_bits()]);
    hasher.update((input_idx as u64).to_le_bytes());

    for input in &tx.inputs {
        hasher.update(input.previous_outpoint.transaction_id.as_bytes());
        hasher.update(input.previous_outpoint.index.to_le_bytes());
        hasher.update(input.sequence.to_le_bytes());
    }

    for output in &tx.outputs {
        hasher.update(output.value.to_le_bytes());
        hasher.update(output.script_public_key.version.to_le_bytes());
        hasher.update(output.script_public_key.script.as_slice());
    }

    hasher.finalize()
}

/// Computes the ECDSA signature hash for a given transaction input.
pub fn calc_ecdsa_signature_hash(
    tx: &Transaction,
    input_idx: usize,
    hash_type: SigHashType,
) -> Hash {
    let mut hasher = TransactionSigningHashECDSA::new();
    hasher.update(tx.version.to_le_bytes());
    hasher.update([hash_type.to_bits()]);
    hasher.update((input_idx as u64).to_le_bytes());

    for input in &tx.inputs {
        hasher.update(input.previous_outpoint.transaction_id.as_bytes());
        hasher.update(input.previous_outpoint.index.to_le_bytes());
        hasher.update(input.sequence.to_le_bytes());
    }

    for output in &tx.outputs {
        hasher.update(output.value.to_le_bytes());
        hasher.update(output.script_public_key.version.to_le_bytes());
        hasher.update(output.script_public_key.script.as_slice());
    }

    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subnets::SubnetworkId;

    #[test]
    fn test_signature_hash_determinism() {
        let tx = Transaction::new(1, vec![], vec![], 0, SubnetworkId::NATIVE, 0, vec![]);
        let hash1 = calc_schnorr_signature_hash(&tx, 0, SigHashType::ALL);
        let hash2 = calc_schnorr_signature_hash(&tx, 0, SigHashType::ALL);
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, Hash::from_bytes([0u8; 32]));
    }
}
