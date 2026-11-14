use crate::tx::{TransactionOutpoint, UtxoEntry};
use crate::utxo::utxo_diff::UtxoDiff;
use borsh::BorshSerialize;
use jio_hashes::Hash;
use jio_muhash::MuHash;

/// Serializes a UTXO entry with its outpoint into canonical byte representation for MuHash.
pub fn serialize_utxo_for_muhash(outpoint: &TransactionOutpoint, entry: &UtxoEntry) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(64);
    outpoint
        .serialize(&mut bytes)
        .expect("serialization succeeds");
    entry.serialize(&mut bytes).expect("serialization succeeds");
    bytes
}

/// Computes the new UTXO commitment after applying a `UtxoDiff` to an existing `MuHash`.
pub fn apply_diff_to_muhash(muhash: &mut MuHash, diff: &UtxoDiff) {
    for (outpoint, entry) in &diff.to_remove {
        let bytes = serialize_utxo_for_muhash(outpoint, entry);
        muhash.remove_element(&bytes);
    }
    for (outpoint, entry) in &diff.to_add {
        let bytes = serialize_utxo_for_muhash(outpoint, entry);
        muhash.add_element(&bytes);
    }
}

/// Finalizes a MuHash to produce a 32-byte UTXO commitment hash.
pub fn finalize_muhash(muhash: &mut MuHash) -> Hash {
    muhash.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_txscript::pay_to_pubkey_script;

    #[test]
    fn test_muhash_diff_roundtrip() {
        let mut muhash = MuHash::new();
        let initial_hash = muhash.finalize();

        let mut diff = UtxoDiff::default();
        let outpoint = TransactionOutpoint::new(Hash::from_bytes([1u8; 32]), 0);
        let entry = UtxoEntry::new(50_000, pay_to_pubkey_script(&[3u8; 32]).into(), 10, false);
        diff.add_entry(outpoint, entry.clone());

        apply_diff_to_muhash(&mut muhash, &diff);
        let updated_hash = muhash.finalize();
        assert_ne!(initial_hash, updated_hash);

        // Reverse diff
        let mut reverse_diff = UtxoDiff::default();
        reverse_diff.remove_entry(outpoint, entry);
        apply_diff_to_muhash(&mut muhash, &reverse_diff);
        let final_hash = muhash.finalize();
        assert_eq!(initial_hash, final_hash);
    }
}
