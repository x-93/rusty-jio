use crate::tx::{Transaction, TransactionId};
use jio_hashes::Hash;
use jio_merkle::calc_merkle_root;

/// Computes the hash Merkle root of all transactions in a block.
pub fn calc_block_hash_merkle_root(transactions: &[Transaction]) -> Hash {
    let hashes: Vec<Hash> = transactions.iter().map(|tx| tx.hash()).collect();
    calc_merkle_root(&hashes)
}

/// Computes the Merkle root of accepted transaction IDs.
pub fn calc_block_accepted_id_merkle_root(accepted_ids: &[TransactionId]) -> Hash {
    calc_merkle_root(accepted_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_merkle_root() {
        let root = calc_block_hash_merkle_root(&[]);
        assert_eq!(root, Hash::from_bytes([0u8; 32]));
    }
}
