use crate::prefix::Prefix;
use bech32::{self, FromBase32, ToBase32, Variant};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddressError {
    #[error("Bech32 encoding/decoding error: {0}")]
    Bech32(#[from] bech32::Error),

    #[error("Invalid address format")]
    InvalidFormat,

    #[error("Unknown prefix: {0}")]
    UnknownPrefix(String),

    #[error("Invalid version byte: {0}")]
    InvalidVersion(u8),
}

/// Address version indicating the underlying key or script type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Version {
    PubKey = 0,      // 32-byte Schnorr public key
    PubKeyECDSA = 1, // 33-byte secp256k1 public key
    ScriptHash = 8,  // P2SH multisig / contract script hash
}

impl Version {
    pub fn from_u8(v: u8) -> Result<Self, AddressError> {
        match v {
            0 => Ok(Version::PubKey),
            1 => Ok(Version::PubKeyECDSA),
            8 => Ok(Version::ScriptHash),
            other => Err(AddressError::InvalidVersion(other)),
        }
    }

    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// A human-readable Bech32m address (e.g., `jio:qp3d...`).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Address {
    pub prefix: Prefix,
    pub version: Version,
    pub payload: Vec<u8>,
}

impl Address {
    pub fn new(prefix: Prefix, version: Version, payload: Vec<u8>) -> Self {
        Self {
            prefix,
            version,
            payload,
        }
    }

    /// Encode address into standard string: `<prefix>:<bech32m_payload>`
    pub fn encode(&self) -> Result<String, AddressError> {
        let mut data = Vec::with_capacity(1 + self.payload.len());
        data.push(self.version as u8);
        data.extend_from_slice(&self.payload);

        let encoded = bech32::encode(self.prefix.as_str(), data.to_base32(), Variant::Bech32m)?;
        if let Some(pos) = encoded.find('1') {
            let (hrp, rest) = encoded.split_at(pos);
            Ok(format!("{}:{}", hrp, &rest[1..]))
        } else {
            Ok(encoded)
        }
    }

    /// Decode address string: `jio:qp...` -> Address
    pub fn decode(addr_str: &str) -> Result<Self, AddressError> {
        let addr_str = addr_str.trim();
        let parts: Vec<&str> = addr_str.split(':').collect();
        if parts.len() != 2 {
            return Err(AddressError::InvalidFormat);
        }
        let prefix = Prefix::from_str(parts[0]).map_err(AddressError::UnknownPrefix)?;

        // Reconstruct standard bech32 string with separator '1'
        let bech32_str = format!("{}1{}", parts[0], parts[1]);
        let (hrp, base32_data, variant) = bech32::decode(&bech32_str)?;

        if variant != Variant::Bech32m || hrp != prefix.as_str() {
            return Err(AddressError::InvalidFormat);
        }

        let raw_bytes: Vec<u8> = Vec::from_base32(&base32_data)?;
        if raw_bytes.is_empty() {
            return Err(AddressError::InvalidFormat);
        }

        let version = Version::from_u8(raw_bytes[0])?;
        let payload = raw_bytes[1..].to_vec();

        Ok(Self {
            prefix,
            version,
            payload,
        })
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.encode() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "<invalid address>"),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let s = self.encode().map_err(serde::ser::Error::custom)?;
            serializer.serialize_str(&s)
        } else {
            (self.prefix, self.version, &self.payload).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::decode(&s).map_err(serde::de::Error::custom)
        } else {
            let (prefix, version, payload) =
                <(Prefix, Version, Vec<u8>)>::deserialize(deserializer)?;
            Ok(Self {
                prefix,
                version,
                payload,
            })
        }
    }
}
