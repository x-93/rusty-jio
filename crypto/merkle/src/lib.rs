use jio_hashes::{Hash, MerkleBranchHash};

/// Combine two hashes using domain-separated MerkleBranchHash
pub fn hash_branch(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = MerkleBranchHash::new();
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    hasher.finalize()
}

/// Compute the Merkle root from an ordered list of leaf hashes (e.g. transaction IDs).
/// Follows the consensus rule: if an odd number of hashes is encountered at any level,
/// the last hash is duplicated to form a complete pair.
pub fn calc_merkle_root(mut hashes: Vec<Hash>) -> Hash {
    if hashes.is_empty() {
        return Hash::ZERO;
    }

    while hashes.len() > 1 {
        let mut next_level = Vec::with_capacity(hashes.len().div_ceil(2));

        for chunk in hashes.chunks(2) {
            if chunk.len() == 2 {
                next_level.push(hash_branch(&chunk[0], &chunk[1]));
            } else {
                // If odd number of hashes, duplicate the last hash
                next_level.push(hash_branch(&chunk[0], &chunk[0]));
            }
        }

        hashes = next_level;
    }

    hashes[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_empty_and_single() {
        assert_eq!(calc_merkle_root(vec![]), Hash::ZERO);

        let h1 = Hash::from_bytes([1u8; 32]);
        assert_eq!(calc_merkle_root(vec![h1]), h1);
    }

    #[test]
    fn test_merkle_root_even_count() {
        let h1 = Hash::from_bytes([1u8; 32]);
        let h2 = Hash::from_bytes([2u8; 32]);

        let expected = hash_branch(&h1, &h2);
        assert_eq!(calc_merkle_root(vec![h1, h2]), expected);
    }

    #[test]
    fn test_merkle_root_odd_count() {
        let h1 = Hash::from_bytes([1u8; 32]);
        let h2 = Hash::from_bytes([2u8; 32]);
        let h3 = Hash::from_bytes([3u8; 32]);

        // Level 1:
        // pair 1: branch(h1, h2)
        // pair 2: branch(h3, h3) - duplicated
        // Level 2:
        // root: branch(branch(h1, h2), branch(h3, h3))
        let level1_0 = hash_branch(&h1, &h2);
        let level1_1 = hash_branch(&h3, &h3);
        let expected_root = hash_branch(&level1_0, &level1_1);

        assert_eq!(calc_merkle_root(vec![h1, h2, h3]), expected_root);
    }

    #[test]
    fn test_merkle_root_four_leaves() {
        let h1 = Hash::from_bytes([1u8; 32]);
        let h2 = Hash::from_bytes([2u8; 32]);
        let h3 = Hash::from_bytes([3u8; 32]);
        let h4 = Hash::from_bytes([4u8; 32]);

        let left = hash_branch(&h1, &h2);
        let right = hash_branch(&h3, &h4);
        let expected = hash_branch(&left, &right);

        assert_eq!(calc_merkle_root(vec![h1, h2, h3, h4]), expected);
    }
}
