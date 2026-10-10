use borsh::{BorshDeserialize, BorshSerialize};
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub const SUBNETWORK_ID_SIZE: usize = 20;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubnetworkId([u8; SUBNETWORK_ID_SIZE]);

impl SubnetworkId {
    pub const fn from_bytes(bytes: [u8; SUBNETWORK_ID_SIZE]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; SUBNETWORK_ID_SIZE] {
        &self.0
    }

    pub const fn is_native(&self) -> bool {
        let mut i = 0;
        while i < SUBNETWORK_ID_SIZE {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    pub const fn is_coinbase(&self) -> bool {
        if self.0[0] != 1 {
            return false;
        }
        let mut i = 1;
        while i < SUBNETWORK_ID_SIZE {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    pub const fn is_registry(&self) -> bool {
        if self.0[0] != 2 {
            return false;
        }
        let mut i = 1;
        while i < SUBNETWORK_ID_SIZE {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    pub const fn is_builtin(&self) -> bool {
        self.is_native() || self.is_coinbase() || self.is_registry()
    }
}

pub const SUBNETWORK_ID_NATIVE: SubnetworkId = SubnetworkId([0; SUBNETWORK_ID_SIZE]);

pub const SUBNETWORK_ID_COINBASE: SubnetworkId = SubnetworkId([
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

pub const SUBNETWORK_ID_REGISTRY: SubnetworkId = SubnetworkId([
    2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

impl AsRef<[u8]> for SubnetworkId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Display for SubnetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl Debug for SubnetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid hex string for subnetwork id")]
pub struct SubnetworkHexError;

impl FromStr for SubnetworkId {
    type Err = SubnetworkHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != SUBNETWORK_ID_SIZE * 2 {
            return Err(SubnetworkHexError);
        }
        let mut bytes = [0u8; SUBNETWORK_ID_SIZE];
        for i in 0..SUBNETWORK_ID_SIZE {
            let high = char_to_hex(s.as_bytes()[i * 2])?;
            let low = char_to_hex(s.as_bytes()[i * 2 + 1])?;
            bytes[i] = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

fn char_to_hex(b: u8) -> Result<u8, SubnetworkHexError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(SubnetworkHexError),
    }
}

impl MemSizeEstimator for SubnetworkId {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
    }
}