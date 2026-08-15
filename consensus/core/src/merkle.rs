use crate::tx::Transaction;
use jio_hashes::{Hash, MerkleBranchHash, ZERO_HASH};

pub fn calc_merkle_root(mut leaves: Vec<Hash>) -> Hash {
    if leaves.is_empty() {
        return ZERO_HASH;
    }

    while leaves.len() > 1 {
        let mut next_level = Vec::with_capacity((leaves.len() + 1) / 2);
        for chunk in leaves.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() > 1 { chunk[1] } else { left };
            let mut hasher = MerkleBranchHash::new();
            hasher.write(left);
            hasher.write(right);
            next_level.push(hasher.finalize());
        }
        leaves = next_level;
    }

    leaves[0]
}

pub fn calc_tx_merkle_root(txs: &[Transaction]) -> Hash {
    if txs.is_empty() {
        return ZERO_HASH;
    }
    let tx_ids: Vec<Hash> = txs.iter().map(crate::hashing::tx::tx_id).collect();
    calc_merkle_root(tx_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_empty() {
        assert_eq!(calc_merkle_root(vec![]), ZERO_HASH);
    }

    #[test]
    fn test_merkle_root_single() {
        let h = Hash::from_bytes([1u8; 32]);
        assert_eq!(calc_merkle_root(vec![h]), h);
    }

    #[test]
    fn test_merkle_root_multiple() {
        let h1 = Hash::from_bytes([1u8; 32]);
        let h2 = Hash::from_bytes([2u8; 32]);
        let root = calc_merkle_root(vec![h1, h2]);
        assert_ne!(root, h1);
        assert_ne!(root, h2);
    }
}
