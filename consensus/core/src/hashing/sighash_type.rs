use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SigHashType(u8);

impl SigHashType {
    pub const ALL: Self = Self(0b00000001);
    pub const NONE: Self = Self(0b00000010);
    pub const SINGLE: Self = Self(0b00000100);
    pub const ANYONECANPAY: Self = Self(0b10000000);

    pub const fn from_u8(val: u8) -> Self {
        Self(val)
    }

    pub const fn to_u8(&self) -> u8 {
        self.0
    }

    pub fn is_all(&self) -> bool {
        self.0 & 0b00000111 == Self::ALL.0
    }

    pub fn is_none(&self) -> bool {
        self.0 & 0b00000111 == Self::NONE.0
    }

    pub fn is_single(&self) -> bool {
        self.0 & 0b00000111 == Self::SINGLE.0
    }

    pub fn is_anyone_can_pay(&self) -> bool {
        self.0 & Self::ANYONECANPAY.0 != 0
    }
}

impl From<u8> for SigHashType {
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl From<SigHashType> for u8 {
    fn from(sig_hash_type: SigHashType) -> Self {
        sig_hash_type.0
    }
}
