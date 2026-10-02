//! Pair of DAA score and timestamp.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct DaaScoreTimestamp {
    pub daa_score: u64,
    pub timestamp: u64,
}

impl DaaScoreTimestamp {
    pub fn new(daa_score: u64, timestamp: u64) -> Self {
        Self { daa_score, timestamp }
    }
}
