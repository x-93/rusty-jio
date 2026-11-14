use crate::{BlueWorkType, hashing};
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::Hash;
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};

pub type ArcHeader = std::sync::Arc<Header>;

/// @category Consensus
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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
    #[serde(with = "blue_work_serde")]
    pub blue_work: BlueWorkType,
    pub blue_score: u64,
    pub pruning_point: Hash,
}

impl Header {
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
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

    /// Finalizes the header and recomputes the header hash
    pub fn finalize(&mut self) {
        self.hash = hashing::header::hash(self);
    }

    /// Computes the pre-PoW hash for this header
    pub fn pre_pow_hash(&self) -> Hash {
        hashing::header::pre_pow_hash(self)
    }

    pub fn direct_parents(&self) -> &[Hash] {
        if self.parents_by_level.is_empty() {
            &[]
        } else {
            &self.parents_by_level[0]
        }
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

pub mod blue_work_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(work: &BlueWorkType, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut hex = [0u8; 48];
            faster_hex::hex_encode(&work.to_be_bytes(), &mut hex)
                .map_err(serde::ser::Error::custom)?;
            let s = std::str::from_utf8(&hex).map_err(serde::ser::Error::custom)?;
            serializer.serialize_str(s)
        } else {
            serde::Serialize::serialize(work, serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BlueWorkType, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String as serde::Deserialize>::deserialize(deserializer)?;
            BlueWorkType::from_hex(&s).map_err(serde::de::Error::custom)
        } else {
            <BlueWorkType as serde::Deserialize>::deserialize(deserializer)
        }
    }
}

/// Calculates the proof of work represented by compact target bits as `BlueWorkType`
pub fn calc_work(bits: u32) -> BlueWorkType {
    let target = jio_math::Uint256::from_compact_target_bits(bits);
    if target.is_zero() {
        return BlueWorkType::ZERO;
    }
    let divisor = if target == jio_math::Uint256::MAX {
        jio_math::Uint256::MAX
    } else {
        target + jio_math::Uint256::from_u64(1)
    };
    let work256 = jio_math::Uint256::MAX / divisor;
    jio_math::Uint192([work256.0[0], work256.0[1], work256.0[2]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_math::Uint192;
    use serde_json::Value;

    #[test]
    fn test_calc_work() {
        let work = calc_work(0x1e7fffff);
        assert!(work > BlueWorkType::ZERO);
        assert_eq!(calc_work(0), BlueWorkType::ZERO);
    }

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
