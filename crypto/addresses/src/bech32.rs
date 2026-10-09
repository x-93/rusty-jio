//! Bech32 and Bech32m address encoding and decoding targeting Jio network prefixes.
//!
//! Supports "jiomain", "jiotest", "jiosim", "jiodev" prefixes with 8-character polymod
//! checksum and colon ':' separator.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

pub const CHARSET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum AddressError {
    #[error("invalid address format, expected <prefix>:<payload>")]
    InvalidFormat,
    #[error("empty prefix or payload")]
    EmptyComponent,
    #[error("invalid character in address: '{0}'")]
    InvalidChar(char),
    #[error("unsupported address prefix: '{0}'")]
    UnsupportedPrefix(String),
    #[error("checksum verification failed")]
    BadChecksum,
    #[error("invalid payload bit conversion")]
    InvalidBitConversion,
    #[error("invalid version byte: {0}")]
    InvalidVersion(u8),
    #[error("invalid payload length: {0}")]
    InvalidPayloadLength(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub enum Prefix {
    Mainnet,
    Testnet,
    Simnet,
    Devnet,
}

impl Prefix {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mainnet => "jiomain",
            Self::Testnet => "jiotest",
            Self::Simnet => "jiosim",
            Self::Devnet => "jiodev",
        }
    }
}

impl Display for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Prefix {
    type Err = AddressError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "jiomain" => Ok(Self::Mainnet),
            "jiotest" => Ok(Self::Testnet),
            "jiosim" => Ok(Self::Simnet),
            "jiodev" => Ok(Self::Devnet),
            _ => Err(AddressError::UnsupportedPrefix(s.to_string())),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "borsh", borsh(use_discriminant = true))]
pub enum Version {
    PubKey = 0,
    PubKeyECDSA = 1,
    ScriptHash = 8,
}

impl TryFrom<u8> for Version {
    type Error = AddressError;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::PubKey),
            1 => Ok(Self::PubKeyECDSA),
            8 => Ok(Self::ScriptHash),
            _ => Err(AddressError::InvalidVersion(v)),
        }
    }
}

/// A parsed Jio blockchain address.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct Address {
    pub prefix: Prefix,
    pub version: Version,
    pub payload: Vec<u8>,
}

impl Address {
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
            payload: payload.to_vec(),
        })
    }

    pub fn encode(&self) -> String {
        encode_address(self.prefix.as_str(), self.version as u8, &self.payload)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Address({})", self.encode())
    }
}

impl FromStr for Address {
    type Err = AddressError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (prefix_str, version_byte, payload) = decode_address(s)?;
        let prefix = Prefix::from_str(&prefix_str)?;
        let version = Version::try_from(version_byte)?;
        Self::new(prefix, version, &payload)
    }
}

// -----------------------------------------------------------------------------
// Low-level Polymod & Bech32 encoding
// -----------------------------------------------------------------------------

const GENERATOR: [u64; 5] = [
    0x98f2bc8e61,
    0x79b76d99e2,
    0xf33e5fb3c4,
    0xae2eabe2a8,
    0x1e4f43e470,
];

fn polymod(values: impl Iterator<Item = u8>) -> u64 {
    let mut chk: u64 = 1;
    for val in values {
        let top = (chk >> 35) as usize;
        chk = ((chk & 0x07ffffffff) << 5) ^ (val as u64);
        for i in 0..5 {
            if ((top >> i) & 1) == 1 {
                chk ^= GENERATOR[i];
            }
        }
    }
    chk ^ 1
}

fn prefix_expand(prefix: &str) -> impl Iterator<Item = u8> + '_ {
    prefix
        .bytes()
        .map(|b| b & 0x1f)
        .chain(std::iter::once(0))
}

pub fn convert_bits(data: &[u8], from: u32, to: u32, pad: bool) -> Result<Vec<u8>, AddressError> {
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut ret = Vec::new();
    let maxv: u32 = (1 << to) - 1;
    let max_acc: u32 = (1 << (from + to - 1)) - 1;

    for &value in data {
        let val = value as u32;
        if (val >> from) != 0 {
            return Err(AddressError::InvalidBitConversion);
        }
        acc = ((acc << from) | val) & max_acc;
        bits += from;
        while bits >= to {
            bits -= to;
            ret.push(((acc >> bits) & maxv) as u8);
        }
    }

    if pad {
        if bits > 0 {
            ret.push(((acc << (to - bits)) & maxv) as u8);
        }
    } else if bits >= from || ((acc << (to - bits)) & maxv) != 0 {
        return Err(AddressError::InvalidBitConversion);
    }

    Ok(ret)
}

pub fn encode_address(prefix: &str, version: u8, payload: &[u8]) -> String {
    let mut data_bytes = Vec::with_capacity(1 + payload.len());
    data_bytes.push(version);
    data_bytes.extend_from_slice(payload);

    let words = convert_bits(&data_bytes, 8, 5, true).expect("bit conversion should succeed");

    // Polymod over prefix_expand || words || 8 zeros
    let checksum = polymod(
        prefix_expand(prefix)
            .chain(words.iter().copied())
            .chain(std::iter::repeat(0).take(8)),
    );

    let mut result = String::with_capacity(prefix.len() + 1 + words.len() + 8);
    result.push_str(prefix);
    result.push(':');

    for &w in &words {
        result.push(CHARSET[w as usize] as char);
    }
    for i in (0..8).rev() {
        let idx = ((checksum >> (5 * i)) & 0x1f) as usize;
        result.push(CHARSET[idx] as char);
    }

    result
}

pub fn decode_address(s: &str) -> Result<(String, u8, Vec<u8>), AddressError> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(AddressError::InvalidFormat);
    }
    let prefix = parts[0].to_lowercase();
    let payload_str = parts[1].to_lowercase();

    if prefix.is_empty() || payload_str.len() < 8 {
        return Err(AddressError::EmptyComponent);
    }

    let mut words = Vec::with_capacity(payload_str.len());
    for c in payload_str.chars() {
        let pos = CHARSET.iter().position(|&ch| ch as char == c);
        match pos {
            Some(idx) => words.push(idx as u8),
            None => return Err(AddressError::InvalidChar(c)),
        }
    }

    let chk = polymod(prefix_expand(&prefix).chain(words.iter().copied()));
    if chk != 0 {
        return Err(AddressError::BadChecksum);
    }

    let data_words = &words[..words.len() - 8];
    let data_bytes = convert_bits(data_words, 5, 8, false)?;
    if data_bytes.is_empty() {
        return Err(AddressError::EmptyComponent);
    }

    let version = data_bytes[0];
    let payload = data_bytes[1..].to_vec();

    Ok((prefix, version, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_encode_decode_roundtrip() {
        let pubkey = [0x42u8; 32];
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, &pubkey).unwrap();
        let encoded = addr.encode();
        assert!(encoded.starts_with("jiomain:"));

        let decoded: Address = encoded.parse().unwrap();
        assert_eq!(addr, decoded);
        assert_eq!(decoded.prefix, Prefix::Mainnet);
        assert_eq!(decoded.version, Version::PubKey);
        assert_eq!(decoded.payload, pubkey);
    }

    #[test]
    fn test_testnet_prefix() {
        let pubkey = [0x77u8; 32];
        let addr = Address::new(Prefix::Testnet, Version::ScriptHash, &pubkey).unwrap();
        let encoded = addr.encode();
        assert!(encoded.starts_with("jiotest:"));

        let decoded: Address = encoded.parse().unwrap();
        assert_eq!(addr, decoded);
    }
}