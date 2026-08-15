use crate::header::Header;
use jio_hashes::{BlockHash, Hash};

pub fn header_hash(header: &Header) -> Hash {
    hash(header)
}

pub fn hash(header: &Header) -> Hash {
    let mut hasher = BlockHash::new();
    hasher.write(&header.version.to_le_bytes());
    hasher.write(&(header.parents_by_level.len() as u64).to_le_bytes());
    for level in &header.parents_by_level {
        hasher.write(&(level.len() as u64).to_le_bytes());
        for parent in level {
            hasher.write(parent);
        }
    }
    hasher.write(header.hash_merkle_root);
    hasher.write(header.accepted_id_merkle_root);
    hasher.write(header.utxo_commitment);
    hasher.write(&header.timestamp.to_le_bytes());
    hasher.write(&header.bits.to_le_bytes());
    hasher.write(&header.nonce.to_le_bytes());
    hasher.write(&header.daa_score.to_le_bytes());
    hasher.write(&header.blue_score.to_le_bytes());
    hasher.write(&header.blue_work.to_le_bytes());
    hasher.write(header.pruning_point);

    hasher.finalize()
}

pub fn pre_pow_hash(header: &Header) -> Hash {
    let mut hasher = BlockHash::new();
    hasher.write(&header.version.to_le_bytes());
    hasher.write(&(header.parents_by_level.len() as u64).to_le_bytes());
    for level in &header.parents_by_level {
        hasher.write(&(level.len() as u64).to_le_bytes());
        for parent in level {
            hasher.write(parent);
        }
    }
    hasher.write(header.hash_merkle_root);
    hasher.write(header.accepted_id_merkle_root);
    hasher.write(header.utxo_commitment);
    hasher.write(&header.timestamp.to_le_bytes());
    hasher.write(&header.bits.to_le_bytes());
    hasher.write(&header.daa_score.to_le_bytes());
    hasher.write(&header.blue_score.to_le_bytes());
    hasher.write(&header.blue_work.to_le_bytes());
    hasher.write(header.pruning_point);

    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_math::Uint192;
    use std::collections::HashSet;

    #[test]
    fn test_header_hashing() {
        let mut header = Header::default();
        let h1 = header_hash(&header);
        header.nonce = 12345;
        let h2 = header_hash(&header);
        assert_ne!(h1, h2);

        // Pre-PoW hash does not include nonce
        let pp1 = pre_pow_hash(&header);
        header.nonce = 99999;
        let pp2 = pre_pow_hash(&header);
        assert_eq!(pp1, pp2);
    }

    #[test]
    fn test_header_hash_collision_resistance_across_all_fields() {
        let base = Header {
            hash: Hash::default(),
            version: 1,
            parents_by_level: vec![vec![Hash::from_le_u64([10, 20, 30, 40])]],
            hash_merkle_root: Hash::from_le_u64([1, 2, 3, 4]),
            accepted_id_merkle_root: Hash::from_le_u64([5, 6, 7, 8]),
            utxo_commitment: Hash::from_le_u64([9, 10, 11, 12]),
            timestamp: 1600000000,
            bits: 0x1e7fffff,
            nonce: 42,
            daa_score: 100,
            blue_score: 100,
            blue_work: Uint192::from(100u64),
            pruning_point: Hash::from_le_u64([13, 14, 15, 16]),
        };

        let base_hash = header_hash(&base);
        let mut seen_hashes = HashSet::new();
        seen_hashes.insert(base_hash);

        let mut check_diff = |modified: Header, name: &str| {
            let h = header_hash(&modified);
            assert!(
                seen_hashes.insert(h),
                "Hash collision detected modifying {name}: {h}"
            );
        };

        // 1. Version
        let mut h = base.clone();
        h.version = 2;
        check_diff(h, "version");

        // 2. Parents
        let mut h = base.clone();
        h.parents_by_level = vec![vec![Hash::from_le_u64([10, 20, 30, 41])]];
        check_diff(h, "parents_by_level hash diff");

        let mut h = base.clone();
        h.parents_by_level = vec![vec![Hash::from_le_u64([10, 20, 30, 40]), Hash::from_le_u64([99, 99, 99, 99])]];
        check_diff(h, "parents_by_level len diff");

        // 3. Merkle roots
        let mut h = base.clone();
        h.hash_merkle_root = Hash::from_le_u64([1, 2, 3, 5]);
        check_diff(h, "hash_merkle_root");

        let mut h = base.clone();
        h.accepted_id_merkle_root = Hash::from_le_u64([5, 6, 7, 9]);
        check_diff(h, "accepted_id_merkle_root");

        // 4. UTXO commitment
        let mut h = base.clone();
        h.utxo_commitment = Hash::from_le_u64([9, 10, 11, 13]);
        check_diff(h, "utxo_commitment");

        // 5. Timestamp
        let mut h = base.clone();
        h.timestamp += 1;
        check_diff(h, "timestamp");

        // 6. Bits
        let mut h = base.clone();
        h.bits = 0x207fffff;
        check_diff(h, "bits");

        // 7. Nonce
        let mut h = base.clone();
        h.nonce = 43;
        check_diff(h, "nonce");

        // 8. DAA score
        let mut h = base.clone();
        h.daa_score = 101;
        check_diff(h, "daa_score");

        // 9. Blue score
        let mut h = base.clone();
        h.blue_score = 101;
        check_diff(h, "blue_score");

        // 10. Blue work
        let mut h = base.clone();
        h.blue_work = Uint192::from(101u64);
        check_diff(h, "blue_work");

        // 11. Pruning point
        let mut h = base.clone();
        h.pruning_point = Hash::from_le_u64([13, 14, 15, 17]);
        check_diff(h, "pruning_point");

        // 12. Multi-perturbation sweeps (1000 variations)
        for i in 1..=1000u64 {
            let mut h = base.clone();
            h.nonce = h.nonce.wrapping_add(i);
            h.timestamp = h.timestamp.wrapping_add(i * 10);
            h.daa_score = h.daa_score.wrapping_add(i);
            check_diff(h, &format!("sweep {i}"));
        }

        assert_eq!(seen_hashes.len(), 1 + 13 + 1000);
    }
}

