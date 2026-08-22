use crate::tx::Transaction;
use jio_hashes::{Hash, HasherBase, MerkleBranchHash};

/// Calculates the Merkle root of an array of hashes using binary tree reduction.
pub fn calc_merkle_root(hashes: &[Hash]) -> Hash {
    if hashes.is_empty() {
        return Hash::from([0u8; 32]);
    }
    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut current = hashes.to_vec();
    while current.len() > 1 {
        let next_capacity = current.len().div_ceil(2);
        let mut next = Vec::with_capacity(next_capacity);

        for chunk in current.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() > 1 { chunk[1] } else { chunk[0] };

            let mut hasher = MerkleBranchHash::new();
            hasher.update(left.as_bytes());
            hasher.update(right.as_bytes());
            next.push(hasher.finalize());
        }
        current = next;
    }

    current[0]
}

/// Calculates the Merkle root from a list of transactions (using their full witness transaction hashes).
pub fn calc_merkle_root_from_transactions(transactions: &[Transaction]) -> Hash {
    let hashes: Vec<Hash> = transactions.iter().map(|tx| tx.hash()).collect();
    calc_merkle_root(&hashes)
}

/// Calculates the transaction IDs Merkle root from a list of transactions.
pub fn calc_tx_ids_merkle_root(transactions: &[Transaction]) -> Hash {
    let ids: Vec<Hash> = transactions.iter().map(|tx| tx.id()).collect();
    calc_merkle_root(&ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_empty_and_single() {
        assert_eq!(calc_merkle_root(&[]), Hash::from([0u8; 32]));

        let h1 = Hash::from([1u8; 32]);
        assert_eq!(calc_merkle_root(&[h1]), h1);
    }

    #[test]
    fn test_merkle_root_even_and_odd() {
        let h1 = Hash::from([1u8; 32]);
        let h2 = Hash::from([2u8; 32]);
        let h3 = Hash::from([3u8; 32]);

        let root_2 = calc_merkle_root(&[h1, h2]);
        let root_3 = calc_merkle_root(&[h1, h2, h3]);

        assert_ne!(root_2, Hash::default());
        assert_ne!(root_3, Hash::default());
        assert_ne!(root_2, root_3);
    }
}
