//! Subnetwork ID identifier for native, coinbase, and specialized subnetworks.

use core::fmt;
use core::str::FromStr;
use jio_utils::{FromHex, ToHex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct SubnetworkId(pub [u8; 20]);

impl SubnetworkId {
    /// Subnetwork ID for native Jio coin transactions (all zeros).
    pub const NATIVE: Self = Self([0u8; 20]);

    /// Subnetwork ID for coinbase transactions (0x01 followed by 19 zeros).
    pub const COINBASE: Self = {
        let mut bytes = [0u8; 20];
        bytes[0] = 1;
        Self(bytes)
    };

    pub fn is_native(&self) -> bool {
        *self == Self::NATIVE
    }

    pub fn is_coinbase(&self) -> bool {
        *self == Self::COINBASE
    }
}

impl fmt::Display for SubnetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_hex())
    }
}

impl fmt::Debug for SubnetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SubnetworkId({})", self)
    }
}

impl FromStr for SubnetworkId {
    type Err = jio_utils::hex::FixedArrayError<20>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let arr = <[u8; 20]>::from_hex(s)?;
        Ok(Self(arr))
    }
}
