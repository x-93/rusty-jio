use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub const SIG_HASH_ALL: u8 = 0b0000_0001;
pub const SIG_HASH_NONE: u8 = 0b0000_0010;
pub const SIG_HASH_SINGLE: u8 = 0b0000_0100;
pub const SIG_HASH_ANYONECANPAY: u8 = 0b1000_0000;
pub const SIG_HASH_MASK: u8 = 0b0000_0111;

/// Represents the signature hash type flags for transaction input signing.
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
pub struct SigHashType(pub u8);

impl SigHashType {
    pub const ALL: Self = Self(SIG_HASH_ALL);
    pub const NONE: Self = Self(SIG_HASH_NONE);
    pub const SINGLE: Self = Self(SIG_HASH_SINGLE);
    pub const ALL_ANYONECANPAY: Self = Self(SIG_HASH_ALL | SIG_HASH_ANYONECANPAY);
    pub const NONE_ANYONECANPAY: Self = Self(SIG_HASH_NONE | SIG_HASH_ANYONECANPAY);
    pub const SINGLE_ANYONECANPAY: Self = Self(SIG_HASH_SINGLE | SIG_HASH_ANYONECANPAY);

    pub fn is_sighash_all(&self) -> bool {
        (self.0 & SIG_HASH_MASK) == SIG_HASH_ALL
    }

    pub fn is_sighash_none(&self) -> bool {
        (self.0 & SIG_HASH_MASK) == SIG_HASH_NONE
    }

    pub fn is_sighash_single(&self) -> bool {
        (self.0 & SIG_HASH_MASK) == SIG_HASH_SINGLE
    }

    pub fn is_anyone_can_pay(&self) -> bool {
        (self.0 & SIG_HASH_ANYONECANPAY) != 0
    }

    pub fn to_u8(&self) -> u8 {
        self.0
    }
}

impl From<u8> for SigHashType {
    fn from(b: u8) -> Self {
        Self(b)
    }
}
