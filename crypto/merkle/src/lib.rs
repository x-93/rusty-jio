// lib.rs
//
// Merkle tree root calculation and proof generation for rusty-jio.
//
// Cryptographic design:
// - Uses `MerkleBranchHash` (domain-separated Blake2b) from `jio-hashes`.
// - Odd-leaf duplication rule (Bitcoin/Kaspa standard): when an odd number of
//   leaves exist at any tree level, the final leaf is paired with itself.
// - Iterative zero-recursion implementation to guarantee O(1) stack space.

use jio_hashes::{Hash, HasherBase, MerkleBranchHash};

/// Computes a parent hash from two child hashes in the Merkle tree.
/// Uses domain-separated `MerkleBranchHash` (Blake2b with "MerkleBranchHash" key).
#[inline(always)]
pub fn hash_merkle_branch(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = MerkleBranchHash::new();
    hasher.update(left.as_bytes());
    hasher.update(right.as_bytes());
    hasher.finalize()
}

/// Calculates the Merkle root of a sequence of leaf hashes.
///
/// If `leaves` is empty, returns `Hash([0u8; 32])`.
/// If `leaves` has 1 item, returns that item directly.
/// If `leaves` has odd count at any level, duplicates the last element.
pub fn calc_merkle_root(leaves: &[Hash]) -> Hash {
    if leaves.is_empty() {
        return Hash::from_bytes([0u8; 32]);
    }
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut current_level = leaves.to_vec();

    while current_level.len() > 1 {
        // If odd, duplicate the last element
        if !current_level.len().is_multiple_of(2) {
            let last = *current_level.last().unwrap();
            current_level.push(last);
        }

        let next_level_len = current_level.len() / 2;
        let mut next_level = Vec::with_capacity(next_level_len);

        for [left, right] in current_level.as_chunks::<2>().0 {
            next_level.push(hash_merkle_branch(left, right));
        }

        current_level = next_level;
    }

    current_level[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_leaves() {
        assert_eq!(calc_merkle_root(&[]), Hash::from_bytes([0u8; 32]));
    }

    #[test]
    fn test_single_leaf() {
        let h = Hash::from_le_u64([1, 2, 3, 4]);
        assert_eq!(calc_merkle_root(&[h]), h);
    }

    #[test]
    fn test_two_leaves() {
        let h1 = Hash::from_le_u64([10, 20, 30, 40]);
        let h2 = Hash::from_le_u64([50, 60, 70, 80]);
        let expected = hash_merkle_branch(&h1, &h2);
        assert_eq!(calc_merkle_root(&[h1, h2]), expected);
    }

    #[test]
    fn test_three_leaves_odd_duplication() {
        let h1 = Hash::from_le_u64([1, 0, 0, 0]);
        let h2 = Hash::from_le_u64([2, 0, 0, 0]);
        let h3 = Hash::from_le_u64([3, 0, 0, 0]);

        // Level 1: [h1, h2], [h3, h3]
        let parent1 = hash_merkle_branch(&h1, &h2);
        let parent2 = hash_merkle_branch(&h3, &h3);
        // Level 2: [parent1, parent2]
        let expected = hash_merkle_branch(&parent1, &parent2);

        assert_eq!(calc_merkle_root(&[h1, h2, h3]), expected);
    }

    #[test]
    fn test_determinism() {
        let leaves: Vec<Hash> = (0..10)
            .map(|i| Hash::from_le_u64([i, i + 1, i + 2, i + 3]))
            .collect();

        let r1 = calc_merkle_root(&leaves);
        let r2 = calc_merkle_root(&leaves);
        assert_eq!(r1, r2);
    }
}
