pub mod hash;
pub mod hasher;

pub use hash::Hash;
pub use hasher::{
    BlockHash, Hasher, HasherBase, MerkleBranchHash, MuHashElementHash, MuHashFinalizeHash,
    PersonalMessageSigningHash, ProofOfWorkHash, TransactionHash, TransactionID,
    TransactionSigningHash, TransactionSigningHashECDSA,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_separation_produces_distinct_hashes() {
        let payload = b"common test payload for block and tx";

        let block_h = BlockHash::hash(payload);
        let tx_h = TransactionHash::hash(payload);
        let tx_id = TransactionID::hash(payload);
        let branch_h = MerkleBranchHash::hash(payload);
        let ecdsa_h = TransactionSigningHashECDSA::hash(payload);

        // Every domain separator must yield a completely distinct 32-byte hash
        assert_ne!(block_h, tx_h);
        assert_ne!(block_h, tx_id);
        assert_ne!(tx_h, tx_id);
        assert_ne!(block_h, branch_h);
        assert_ne!(block_h, ecdsa_h);
        assert_ne!(block_h, Hash::ZERO);
        assert_ne!(ecdsa_h, Hash::ZERO);
    }

    #[test]
    fn test_streaming_update_matches_one_shot_hash() {
        let data = b"streaming incremental data test";

        let one_shot = BlockHash::hash(data);

        let mut hasher = BlockHash::new();
        hasher.update(&data[..10]);
        hasher.update(&data[10..]);
        let incremental = hasher.finalize();

        assert_eq!(one_shot, incremental);
    }

    #[test]
    fn test_hash_hex_serde_roundtrip() {
        let original = BlockHash::hash(b"serialization target");
        let json = serde_json::to_string(&original).unwrap();

        let deserialized: Hash = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);

        // Test with 0x prefix
        let with_prefix = format!("\"0x{}\"", original);
        let de_prefix: Hash = serde_json::from_str(&with_prefix).unwrap();
        assert_eq!(original, de_prefix);
    }
}
