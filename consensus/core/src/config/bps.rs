use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Represents the consensus block rate in blocks per second (BPS).
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
pub struct Bps(pub u64);

impl Bps {
    pub const fn new(bps: u64) -> Self {
        Self(bps)
    }

    /// Target block delay interval in milliseconds.
    pub fn target_time_per_block_ms(&self) -> u64 {
        1000u64.checked_div(self.0).unwrap_or(1000)
    }

    pub const fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Bps {
    fn from(val: u64) -> Self {
        Self::new(val)
    }
}
