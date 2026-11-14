// lib.rs
//
// Public address and prefix handling for rusty-jio.
// Supports Pay-to-Pubkey (P2PK), Pay-to-Script-Hash (P2SH), and Schnorr/ECDSA addresses.

pub mod bech32;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smallvec::SmallVec;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::str::FromStr;
use thiserror::Error;

pub use bech32::Bech32Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AddressError {
    #[error("Bech32 error: {0}")]
    Bech32(#[from] Bech32Error),
    #[error("Unknown prefix: {0}")]
    UnknownPrefix(String),
    #[error("Invalid version byte: {0}")]
    InvalidVersion(u8),
    #[error("Invalid address payload length: {0}")]
    InvalidPayloadLength(usize),
}

/// Network identifier embedded in address prefix
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub enum Prefix {
    Mainnet,
    Testnet,
    Simnet,
    Devnet,
}

impl Prefix {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Prefix::Mainnet => "jio",
            Prefix::Testnet => "jiotest",
            Prefix::Simnet => "jiosim",
            Prefix::Devnet => "jiodev",
        }
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Prefix {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "jio" => Ok(Prefix::Mainnet),
            "jiotest" => Ok(Prefix::Testnet),
            "jiosim" => Ok(Prefix::Simnet),
            "jiodev" => Ok(Prefix::Devnet),
            _ => Err(AddressError::UnknownPrefix(s.to_string())),
        }
    }
}

/// Address type / script version
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
#[borsh(use_discriminant = true)]
#[repr(u8)]
pub enum Version {
    PubKey = 0,
    PubKeyECDSA = 1,
    ScriptHash = 8,
}

impl TryFrom<u8> for Version {
    type Error = AddressError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Version::PubKey),
            1 => Ok(Version::PubKeyECDSA),
            8 => Ok(Version::ScriptHash),
            _ => Err(AddressError::InvalidVersion(val)),
        }
    }
}

/// Strongly-typed network address
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Address {
    pub prefix: Prefix,
    pub version: Version,
    pub payload: SmallVec<[u8; 34]>,
}

impl Address {
    /// Constructs a new Address with payload length check
    pub fn new(prefix: Prefix, version: Version, payload: &[u8]) -> Result<Self, AddressError> {
        match version {
            Version::PubKey => {
                if payload.len() != 32 {
                    return Err(AddressError::InvalidPayloadLength(payload.len()));
                }
            }
            Version::PubKeyECDSA => {
                if payload.len() != 33 {
                    return Err(AddressError::InvalidPayloadLength(payload.len()));
                }
            }
            Version::ScriptHash => {
                if payload.len() != 32 {
                    return Err(AddressError::InvalidPayloadLength(payload.len()));
                }
            }
        }

        Ok(Self {
            prefix,
            version,
            payload: SmallVec::from_slice(payload),
        })
    }

    /// Serializes to human-readable Bech32 string
    pub fn encode_bech32(&self) -> String {
        let mut raw_bytes = Vec::with_capacity(1 + self.payload.len());
        raw_bytes.push(self.version as u8);
        raw_bytes.extend_from_slice(&self.payload);
        bech32::encode(self.prefix.as_str(), &raw_bytes)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode_bech32())
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix_str, raw_bytes) = bech32::decode(s)?;
        if raw_bytes.is_empty() {
            return Err(AddressError::InvalidPayloadLength(0));
        }

        let prefix = Prefix::from_str(&prefix_str)?;
        let version = Version::try_from(raw_bytes[0])?;
        let payload = &raw_bytes[1..];

        Address::new(prefix, version, payload)
    }
}

// Canonical Serde serialization as human-readable Bech32 string
impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.encode_bech32())
        } else {
            Serialize::serialize(
                &(self.prefix, self.version, self.payload.as_slice()),
                serializer,
            )
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String as Deserialize>::deserialize(deserializer)?;
            Address::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            let (prefix, version, payload): (Prefix, Version, Vec<u8>) =
                Deserialize::deserialize(deserializer)?;
            Address::new(prefix, version, &payload).map_err(serde::de::Error::custom)
        }
    }
}

// Borsh binary serialization
impl BorshSerialize for Address {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.prefix, writer)?;
        BorshSerialize::serialize(&self.version, writer)?;
        BorshSerialize::serialize(&(self.payload.len() as u8), writer)?;
        writer.write_all(&self.payload)?;
        Ok(())
    }
}

impl BorshDeserialize for Address {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let prefix = Prefix::deserialize_reader(reader)?;
        let version = Version::deserialize_reader(reader)?;
        let len = u8::deserialize_reader(reader)? as usize;
        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload)?;
        Address::new(prefix, version, &payload)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_p2pk_roundtrip() {
        let pubkey_bytes = [7u8; 32];
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, &pubkey_bytes).unwrap();
        let addr_str = addr.to_string();

        assert!(addr_str.starts_with("jio:"));

        let decoded: Address = addr_str.parse().expect("Address parse must succeed");
        assert_eq!(addr, decoded);
        assert_eq!(decoded.prefix, Prefix::Mainnet);
        assert_eq!(decoded.version, Version::PubKey);
        assert_eq!(decoded.payload.as_slice(), &pubkey_bytes);
    }

    #[test]
    fn test_address_testnet_p2sh() {
        let scripthash = [0xabu8; 32];
        let addr = Address::new(Prefix::Testnet, Version::ScriptHash, &scripthash).unwrap();
        let addr_str = addr.to_string();

        assert!(addr_str.starts_with("jiotest:"));

        let decoded: Address = addr_str.parse().expect("Address parse must succeed");
        assert_eq!(addr, decoded);
        assert_eq!(decoded.prefix, Prefix::Testnet);
        assert_eq!(decoded.version, Version::ScriptHash);
    }

    #[test]
    fn test_invalid_length_rejection() {
        let invalid_len_pubkey = [1u8; 15]; // Only 15 bytes instead of 32
        assert!(Address::new(Prefix::Mainnet, Version::PubKey, &invalid_len_pubkey).is_err());
    }

    #[test]
    fn test_serde_json_roundtrip() {
        let pubkey_bytes = [42u8; 32];
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, &pubkey_bytes).unwrap();
        let json_str = serde_json::to_string(&addr).unwrap();
        let de_addr: Address = serde_json::from_str(&json_str).unwrap();
        assert_eq!(addr, de_addr);
    }

    #[test]
    fn test_borsh_roundtrip() {
        let pubkey_bytes = [99u8; 32];
        let addr = Address::new(Prefix::Devnet, Version::PubKey, &pubkey_bytes).unwrap();
        let mut encoded = Vec::new();
        BorshSerialize::serialize(&addr, &mut encoded).unwrap();
        let de_addr: Address =
            BorshDeserialize::deserialize_reader(&mut encoded.as_slice()).unwrap();
        assert_eq!(addr, de_addr);
    }
}
