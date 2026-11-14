// blockstatus.rs
//
// Block validation status bitflags in the BlockDAG.

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct BlockStatus(u8);

impl BlockStatus {
    pub const STATUS_INVALID: Self = Self(0);
    pub const STATUS_HEADER_ONLY: Self = Self(1 << 0);
    pub const STATUS_UTXO_VALID: Self = Self(1 << 1);
    pub const STATUS_DISQUALIFIED: Self = Self(1 << 2);

    #[inline(always)]
    pub const fn new(flags: u8) -> Self {
        Self(flags)
    }

    #[inline(always)]
    pub fn is_header_only(&self) -> bool {
        self.0 & Self::STATUS_HEADER_ONLY.0 != 0
    }

    #[inline(always)]
    pub fn is_utxo_valid(&self) -> bool {
        self.0 & Self::STATUS_UTXO_VALID.0 != 0
    }

    #[inline(always)]
    pub fn is_disqualified(&self) -> bool {
        self.0 & Self::STATUS_DISQUALIFIED.0 != 0
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.is_utxo_valid() && !self.is_disqualified()
    }
}
