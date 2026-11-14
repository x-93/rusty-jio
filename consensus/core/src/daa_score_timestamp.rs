use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct DaaScoreTimestamp {
    pub daa_score: u64,
    pub timestamp: u64,
}

impl DaaScoreTimestamp {
    #[inline(always)]
    pub const fn new(daa_score: u64, timestamp: u64) -> Self {
        Self {
            daa_score,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daa_score_timestamp_ordering() {
        let a = DaaScoreTimestamp::new(100, 1000);
        let b = DaaScoreTimestamp::new(101, 900);
        assert!(a < b);
    }
}
