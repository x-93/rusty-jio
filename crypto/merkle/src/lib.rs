//! Standard transaction Merkle tree root computation and proof verification.

use jio_hashes::{Hash, MerkleBranchHash, ZERO_HASH};

/// Compute standard transaction Merkle tree root from a list of hashes.
/// If empty, returns `ZERO_HASH`.
/// If odd number of leaves at any level, the last element is duplicated.
pub fn calc_merkle_root(leaves: &[Hash]) -> Hash {
    if leaves.is_empty() {
        return ZERO_HASH;
    }
    if leaves.len() == 1 {
        return leaves[0];
    }

    let mut current_level = leaves.to_vec();
    while current_level.len() > 1 {
        let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));
        for chunk in current_level.chunks(2) {
            let left = chunk[0];
            let right = if chunk.len() > 1 { chunk[1] } else { left };
            let mut hasher = MerkleBranchHash::new();
            hasher.update(left.as_bytes());
            hasher.update(right.as_bytes());
            next_level.push(hasher.finalize());
        }
        current_level = next_level;
    }
    current_level[0]
}

/// A full Merkle tree storing all intermediate levels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleTree {
    levels: Vec<Vec<Hash>>,
}

impl MerkleTree {
    pub fn new(leaves: &[Hash]) -> Self {
        if leaves.is_empty() {
            return Self {
                levels: vec![vec![ZERO_HASH]],
            };
        }

        let mut levels = Vec::new();
        let mut current_level = leaves.to_vec();
        levels.push(current_level.clone());

        while current_level.len() > 1 {
            let mut next_level = Vec::with_capacity(current_level.len().div_ceil(2));
            for chunk in current_level.chunks(2) {
                let left = chunk[0];
                let right = if chunk.len() > 1 { chunk[1] } else { left };
                let mut hasher = MerkleBranchHash::new();
                hasher.update(left.as_bytes());
                hasher.update(right.as_bytes());
                next_level.push(hasher.finalize());
            }
            levels.push(next_level.clone());
            current_level = next_level;
        }

        Self { levels }
    }

    pub fn root(&self) -> Hash {
        self.levels
            .last()
            .and_then(|lvl| lvl.first())
            .copied()
            .unwrap_or(ZERO_HASH)
    }

    pub fn generate_proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        let leaves = self.levels.first()?;
        if leaf_index >= leaves.len() {
            return None;
        }

        let mut path = Vec::new();
        let mut idx = leaf_index;

        for level in &self.levels[..self.levels.len() - 1] {
            let is_right = idx % 2 == 1;
            let sibling_idx = if is_right {
                idx - 1
            } else if idx + 1 < level.len() {
                idx + 1
            } else {
                idx // Duplicate last odd element
            };
            path.push((level[sibling_idx], is_right));
            idx /= 2;
        }

        Some(MerkleProof { path })
    }
}

/// Merkle inclusion proof for a leaf.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct MerkleProof {
    /// List of (sibling_hash, is_current_node_on_the_right)
    pub path: Vec<(Hash, bool)>,
}

impl MerkleProof {
    pub fn verify(&self, root: &Hash, leaf: &Hash) -> bool {
        let mut current = *leaf;
        for &(sibling, is_right) in &self.path {
            let mut hasher = MerkleBranchHash::new();
            if is_right {
                hasher.update(sibling.as_bytes());
                hasher.update(current.as_bytes());
            } else {
                hasher.update(current.as_bytes());
                hasher.update(sibling.as_bytes());
            }
            current = hasher.finalize();
        }
        current == *root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_empty_and_single() {
        assert_eq!(calc_merkle_root(&[]), ZERO_HASH);
        let h1 = Hash::from_u64_word(42);
        assert_eq!(calc_merkle_root(&[h1]), h1);
    }

    #[test]
    fn test_merkle_tree_proof_verification() {
        let leaves: Vec<Hash> = (0..5).map(|i| Hash::from_u64_word(i * 100 + 1)).collect();
        let tree = MerkleTree::new(&leaves);
        let root = tree.root();
        assert_eq!(root, calc_merkle_root(&leaves));

        for (i, leaf) in leaves.iter().enumerate() {
            let proof = tree.generate_proof(i).expect("valid proof");
            assert!(proof.verify(&root, leaf));
            // Invalid leaf should fail verification
            let bad_leaf = Hash::from_u64_word(99999);
            assert!(!proof.verify(&root, &bad_leaf));
        }
    }
}
