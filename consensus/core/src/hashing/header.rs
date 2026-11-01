use super::HasherExtensions;
use crate::header::Header;
use jio_hashes::Hash;

/// Returns the header hash using the provided nonce+timestamp instead of those in the header.
#[inline]
pub fn hash_override_nonce_time(header: &Header, nonce: u64, timestamp: u64) -> Hash {
    let mut hasher = jio_hashes::BlockHash::new();
    hasher
        .update(header.version.to_le_bytes())
        .write_len(header.parents_by_level.len()); // Write the number of parent levels

    // Write parents at each level
    header.parents_by_level.iter().for_each(|level| {
        hasher.write_var_array(level);
    });

    // Write all header fields
    hasher
        .update(header.hash_merkle_root)
        .update(header.accepted_id_merkle_root)
        .update(header.utxo_commitment)
        .update(timestamp.to_le_bytes())
        .update(header.bits.to_le_bytes())
        .update(nonce.to_le_bytes())
        .update(header.daa_score.to_le_bytes())
        .update(header.blue_score.to_le_bytes())
        .write_blue_work(header.blue_work)
        .update(header.pruning_point);

    hasher.finalize()
}

/// Returns the header hash.
pub fn hash(header: &Header) -> Hash {
    hash_override_nonce_time(header, header.nonce, header.timestamp)
}

/// Returns the pre-PoW commitment hash (with timestamp=0 and nonce=0) using ProofOfWorkHash.
#[inline]
pub fn pre_pow_hash(header: &Header) -> Hash {
    let mut hasher = jio_hashes::ProofOfWorkHash::new();
    hasher
        .update(header.version.to_le_bytes())
        .write_len(header.parents_by_level.len()); // Write the number of parent levels

    // Write parents at each level
    header.parents_by_level.iter().for_each(|level| {
        hasher.write_var_array(level);
    });

    // Write all header fields with timestamp=0 and nonce=0
    hasher
        .update(header.hash_merkle_root)
        .update(header.accepted_id_merkle_root)
        .update(header.utxo_commitment)
        .update(0u64.to_le_bytes())
        .update(header.bits.to_le_bytes())
        .update(0u64.to_le_bytes())
        .update(header.daa_score.to_le_bytes())
        .update(header.blue_score.to_le_bytes())
        .write_blue_work(header.blue_work)
        .update(header.pruning_point);

    hasher.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlueWorkType, blockhash};

    #[test]
    fn test_header_hashing() {
        let header = Header::new_finalized(
            1,
            vec![vec![1.into()]],
            Default::default(),
            Default::default(),
            Default::default(),
            234,
            23,
            567,
            0,
            0.into(),
            0,
            Default::default(),
        );
        assert_ne!(blockhash::NONE, header.hash);
        assert_ne!(header.hash, pre_pow_hash(&header));
    }

    #[test]
    fn test_hash_blue_work() {
        let tests: Vec<(BlueWorkType, Vec<u8>)> = vec![
            (0.into(), vec![0, 0, 0, 0, 0, 0, 0, 0]),
            (123456.into(), vec![3, 0, 0, 0, 0, 0, 0, 0, 1, 226, 64]),
        ];

        for test in tests {
            let mut hasher = jio_hashes::BlockHash::new();
            hasher.write_blue_work(test.0);

            let mut hasher2 = jio_hashes::BlockHash::new();
            hasher2.update(test.1);
            assert_eq!(hasher.finalize(), hasher2.finalize())
        }
    }
}
