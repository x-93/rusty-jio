use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SigHashType(u8);

impl SigHashType {
    pub const ALL: Self = Self(0b0000_0001);
    pub const NONE: Self = Self(0b0000_0010);
    pub const SINGLE: Self = Self(0b0000_0100);
    pub const ANYONECANPAY: Self = Self(0b1000_0000);

    pub const fn from_u8(val: u8) -> Self {
        Self(val)
    }

    pub const fn to_u8(self) -> u8 {
        self.0
    }

    pub fn is_all(self) -> bool {
        (self.0 & 0b0000_0111) == Self::ALL.0
    }

    pub fn is_none(self) -> bool {
        (self.0 & 0b0000_0111) == Self::NONE.0
    }

    pub fn is_single(self) -> bool {
        (self.0 & 0b0000_0111) == Self::SINGLE.0
    }

    pub fn is_anyone_can_pay(self) -> bool {
        (self.0 & Self::ANYONECANPAY.0) != 0
    }
}

impl Default for SigHashType {
    fn default() -> Self {
        Self::ALL
    }
}

impl From<u8> for SigHashType {
    fn from(val: u8) -> Self {
        Self(val)
    }
}

impl From<SigHashType> for u8 {
    fn from(sht: SigHashType) -> Self {
        sht.0
    }
}
