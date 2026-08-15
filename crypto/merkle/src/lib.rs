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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root() {
        let h1 = Hash::from_bytes([1u8; 32]);
        let h2 = Hash::from_bytes([2u8; 32]);
        let root = calc_merkle_root(vec![h1, h2]);
        assert_ne!(root, ZERO_HASH);
    }
}
