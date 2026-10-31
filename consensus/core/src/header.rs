use crate::{BlueWorkType, hashing};
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::Hash;
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};
use std::mem::size_of;

/// @category Consensus
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    /// Cached hash
    pub hash: Hash,
    pub version: u16,
    pub parents_by_level: Vec<Vec<Hash>>,
    pub hash_merkle_root: Hash,
    pub accepted_id_merkle_root: Hash,
    pub utxo_commitment: Hash,
    /// Timestamp is in milliseconds
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub blue_work: BlueWorkType,
    pub blue_score: u64,
    pub pruning_point: Hash,
}

impl Header {
    #[allow(clippy::too_many_arguments)]
    pub fn new_finalized(
        version: u16,
        parents_by_level: Vec<Vec<Hash>>,
        hash_merkle_root: Hash,
        accepted_id_merkle_root: Hash,
        utxo_commitment: Hash,
        timestamp: u64,
        bits: u32,
        nonce: u64,
        daa_score: u64,
        blue_work: BlueWorkType,
        blue_score: u64,
        pruning_point: Hash,
    ) -> Self {
        let mut header = Self {
            hash: Default::default(), // Temp init before the finalize below
            version,
            parents_by_level,
            hash_merkle_root,
            accepted_id_merkle_root,
            utxo_commitment,
            nonce,
            timestamp,
            daa_score,
            bits,
            blue_work,
            blue_score,
            pruning_point,
        };
        header.finalize();
        header
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u16,
        parents_by_level: Vec<Vec<Hash>>,
        hash_merkle_root: Hash,
        accepted_id_merkle_root: Hash,
        utxo_commitment: Hash,
        timestamp: u64,
        bits: u32,
        nonce: u64,
        daa_score: u64,
        blue_work: BlueWorkType,
        blue_score: u64,
        pruning_point: Hash,
    ) -> Self {
        Self::new_finalized(
            version,
            parents_by_level,
            hash_merkle_root,
            accepted_id_merkle_root,
            utxo_commitment,
            timestamp,
            bits,
            nonce,
            daa_score,
            blue_work,
            blue_score,
            pruning_point,
        )
    }

    /// Finalizes the header and recomputes the header hash
    pub fn finalize(&mut self) {
        self.hash = hashing::header::hash(self);
    }

    pub fn direct_parents(&self) -> &[Hash] {
        if self.parents_by_level.is_empty() {
            &[]
        } else {
            &self.parents_by_level[0]
        }
    }

    /// Check if this header is a genesis block header (no parents).
    pub fn is_genesis(&self) -> bool {
        self.parents_by_level.is_empty() || self.parents_by_level[0].is_empty()
    }

    /// Computes the domain-separated BlockHash over all header fields.
    pub fn compute_hash(&self) -> Hash {
        hashing::header::hash(self)
    }

    /// Computes the PrePowHash (hash of header with timestamp=0 and nonce=0).
    pub fn compute_pre_pow_hash(&self) -> Hash {
        let mut hasher = jio_hashes::ProofOfWorkHash::new();
        hasher.update(self.version.to_le_bytes());

        let num_levels = self.parents_by_level.len() as u32;
        hasher.update(num_levels.to_le_bytes());
        for level in &self.parents_by_level {
            let num_parents = level.len() as u32;
            hasher.update(num_parents.to_le_bytes());
            for parent in level {
                hasher.update(parent.as_bytes());
            }
        }

        hasher.update(self.hash_merkle_root.as_bytes());
        hasher.update(self.accepted_id_merkle_root.as_bytes());
        hasher.update(self.utxo_commitment.as_bytes());

        // Zero timestamp and zero nonce for pre-pow commitments
        hasher.update(0u64.to_le_bytes());
        hasher.update(self.bits.to_le_bytes());
        hasher.update(0u64.to_le_bytes());

        hasher.update(self.daa_score.to_le_bytes());
        let work_bytes = self.blue_work.to_le_bytes();
        hasher.update(work_bytes);
        hasher.update(self.blue_score.to_le_bytes());
        hasher.update(self.pruning_point.as_bytes());

        hasher.finalize()
    }

    /// WARNING: To be used for test purposes only
    pub fn from_precomputed_hash(hash: Hash, parents: Vec<Hash>) -> Header {
        Header {
            version: crate::constants::BLOCK_VERSION,
            hash,
            parents_by_level: vec![parents],
            hash_merkle_root: Default::default(),
            accepted_id_merkle_root: Default::default(),
            utxo_commitment: Default::default(),
            nonce: 0,
            timestamp: 0,
            daa_score: 0,
            bits: 0,
            blue_work: 0.into(),
            blue_score: 0,
            pruning_point: Default::default(),
        }
    }
}

impl AsRef<Header> for Header {
    fn as_ref(&self) -> &Header {
        self
    }
}

impl MemSizeEstimator for Header {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self.parents_by_level.iter().map(|l| l.len()).sum::<usize>() * size_of::<Hash>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_math::Uint192;
    use serde_json::Value;

    #[test]
    fn test_header_ser() {
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
            Uint192([0x1234567890abcfed, 0xc0dec0ffeec0ffee, 0x1234567890abcdef]),
            u64::MAX,
            Default::default(),
        );
        let json = serde_json::to_string(&header).unwrap();
        println!("{}", json);

        let v = serde_json::from_str::<Value>(&json).unwrap();
        let blue_work = v.get("blueWork").expect("missing `blueWork` property");
        let blue_work = blue_work.as_str().expect("`blueWork` is not a string");
        assert_eq!(
            blue_work,
            "1234567890abcdefc0dec0ffeec0ffee1234567890abcfed"
        );
        let blue_score = v.get("blueScore").expect("missing `blueScore` property");
        let blue_score: u64 = blue_score
            .as_u64()
            .expect("blueScore is not a u64 compatible value");
        assert_eq!(blue_score, u64::MAX);

        let h = serde_json::from_str::<Header>(&json).unwrap();
        assert!(h.blue_score == header.blue_score && h.blue_work == header.blue_work);
    }
}
