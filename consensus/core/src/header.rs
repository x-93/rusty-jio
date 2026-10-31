use crate::BlueWorkType;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use jio_hashes::{BlockHash, Hash, ProofOfWorkHash};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub hash: Hash, // Cached hash
    pub version: u16,
    pub parents_by_level: Vec<Vec<Hash>>,
    pub hash_merkle_root: Hash,
    pub accepted_id_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64, // Timestamp is in milliseconds
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub blue_work: BlueWorkType,
    pub blue_score: u64,
    pub pruning_point: Hash,
}

impl Header {
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
        let mut header = Self {
            hash: Hash::ZERO,
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

    /// Finalizes the header and recomputes the header hash
    pub fn finalize(&mut self) {
        self.hash = self.compute_hash();
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
        let mut hasher = BlockHash::new();
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
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.bits.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.daa_score.to_le_bytes());

        let work_bytes = self.blue_work.to_le_bytes();
        hasher.update(work_bytes);
        hasher.update(self.blue_score.to_le_bytes());
        hasher.update(self.pruning_point.as_bytes());

        hasher.finalize()
    }

    /// Computes the PrePowHash (hash of header with timestamp=0 and nonce=0).
    pub fn compute_pre_pow_hash(&self) -> Hash {
        let mut hasher = ProofOfWorkHash::new();
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
