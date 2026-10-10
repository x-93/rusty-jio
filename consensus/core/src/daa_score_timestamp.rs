use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct DaaScoreTimestamp {
    pub daa_score: u64,
    pub timestamp: u64,
}

impl DaaScoreTimestamp {
    pub const fn new(daa_score: u64, timestamp: u64) -> Self {
        Self {
            daa_score,
            timestamp,
        }
    }
}