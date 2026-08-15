use serde::{Deserialize, Serialize};

pub const HARDENED_FLAG: u32 = 0x80000000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChildNumber(pub u32);

impl ChildNumber {
    pub fn normal(index: u32) -> Self {
        assert!(index < HARDENED_FLAG, "index exceeds normal range");
        Self(index)
    }

    pub fn hardened(index: u32) -> Self {
        assert!(index < HARDENED_FLAG, "index exceeds normal range");
        Self(index | HARDENED_FLAG)
    }

    pub fn is_hardened(&self) -> bool {
        (self.0 & HARDENED_FLAG) != 0
    }

    pub fn index(&self) -> u32 {
        self.0 & !HARDENED_FLAG
    }
}
