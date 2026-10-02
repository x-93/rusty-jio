//! # Jio Addresses
//!
//! Bech32 address format with custom BCH error-correcting code for the Jio network.

use core::fmt;
use core::str::FromStr;
use smallvec::SmallVec;

pub mod bech32;
pub use bech32::AddressError;

/// Network prefixes for Jio addresses.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub enum Prefix {
    #[default]
    Jio,
    JioTest,
    JioSim,
    JioDev,
}

impl Prefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Jio => "jio",
            Self::JioTest => "jiotest",
            Self::JioSim => "jiosim",
            Self::JioDev => "jiodev",
        }
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Prefix {
    type Err = AddressError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "jio" => Ok(Self::Jio),
            "jiotest" => Ok(Self::JioTest),
            "jiosim" => Ok(Self::JioSim),
            "jiodev" => Ok(Self::JioDev),
            other => Err(AddressError::InvalidPrefix(other.to_string())),
        }
    }
}

/// Address version indicating script type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "borsh", borsh(use_discriminant = true))]
#[repr(u8)]
pub enum Version {
    #[default]
    PubKey = 0x00,
    PubKeyECDSA = 0x01,
    ScriptHash = 0x08,
}

impl Version {
    pub fn from_u8(v: u8) -> Result<Self, AddressError> {
        match v {
            0x00 => Ok(Self::PubKey),
            0x01 => Ok(Self::PubKeyECDSA),
            0x08 => Ok(Self::ScriptHash),
            _ => Err(AddressError::InvalidVersion(v)),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Jio Bech32 address.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address {
    pub prefix: Prefix,
    pub version: Version,
    pub payload: SmallVec<[u8; 32]>,
}

impl Address {
    pub fn new(prefix: Prefix, version: Version, payload: &[u8]) -> Self {
        Self {
            prefix,
            version,
            payload: SmallVec::from_slice(payload),
        }
    }

    pub fn from_public_key(prefix: Prefix, pubkey: &[u8; 32]) -> Self {
        Self::new(prefix, Version::PubKey, pubkey)
    }

    pub fn from_public_key_ecdsa(prefix: Prefix, pubkey: &[u8; 33]) -> Self {
        Self::new(prefix, Version::PubKeyECDSA, pubkey)
    }

    pub fn from_script_hash(prefix: Prefix, script_hash: &[u8; 32]) -> Self {
        Self::new(prefix, Version::ScriptHash, script_hash)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for Address {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.prefix.serialize(writer)?;
        self.version.serialize(writer)?;
        let bytes: &[u8] = self.payload.as_slice();
        bytes.serialize(writer)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for Address {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let prefix = Prefix::deserialize_reader(reader)?;
        let version = Version::deserialize_reader(reader)?;
        let payload: Vec<u8> = Vec::deserialize_reader(reader)?;
        Ok(Self::new(prefix, version, &payload))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = bech32::encode(self.prefix.as_str(), self.version.to_u8(), &self.payload)
            .map_err(|_| fmt::Error)?;
        write!(f, "{}", s)
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Address({})", self)
    }
}

impl FromStr for Address {
    type Err = AddressError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix_str, version_u8, payload) = bech32::decode(s)?;
        let prefix = Prefix::from_str(&prefix_str)?;
        let version = Version::from_u8(version_u8)?;
        Ok(Self {
            prefix,
            version,
            payload: SmallVec::from_vec(payload),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_roundtrip() {
        let original = Address::new(Prefix::Jio, Version::PubKey, &[0x55; 32]);
        let s = original.to_string();
        assert!(s.starts_with("jio:"));

        let parsed: Address = s.parse().unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_testnet_address_roundtrip() {
        let original = Address::new(Prefix::JioTest, Version::ScriptHash, &[0xaa; 32]);
        let s = original.to_string();
        assert!(s.starts_with("jiotest:"));

        let parsed: Address = s.parse().unwrap();
        assert_eq!(original, parsed);
    }
}
