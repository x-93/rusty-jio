use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

pub mod bech32;

/// Identifies the target network prefix for an address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, AddressError> {
        <Self as std::str::FromStr>::from_str(s)
    }
}

impl std::str::FromStr for Prefix {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "jio" => Ok(Prefix::Mainnet),
            "jiotest" => Ok(Prefix::Testnet),
            "jiosim" => Ok(Prefix::Simnet),
            "jiodev" => Ok(Prefix::Devnet),
            _ => Err(AddressError::UnknownPrefix(s.to_string())),
        }
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AddressError {
    #[error("Decoding error: invalid character '{0}'")]
    DecodingError(char),

    #[error("Bad address payload length")]
    BadPayload,

    #[error("Bad checksum size")]
    BadChecksumSize,

    #[error("Bad checksum")]
    BadChecksum,

    #[error("Unknown prefix: {0}")]
    UnknownPrefix(String),

    #[error("Invalid version byte: {0}")]
    InvalidVersion(u8),

    #[error("Invalid address format")]
    InvalidFormat,
}

/// Address version indicating the underlying key or script type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Version {
    PubKey = 0,      // 32-byte Schnorr public key
    PubKeyECDSA = 1, // 33-byte secp256k1 public key
    ScriptHash = 8,  // P2SH multisig / contract script hash
}

impl TryFrom<u8> for Version {
    type Error = AddressError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Version::PubKey),
            1 => Ok(Version::PubKeyECDSA),
            8 => Ok(Version::ScriptHash),
            other => Err(AddressError::InvalidVersion(other)),
        }
    }
}

impl Version {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// A human-readable Bech32m / CashAddr address (e.g., `jio:qp3d...`).
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

    /// Encode address into standard string: `<prefix>:<bech32_payload>`
    pub fn encode(&self) -> String {
        format!("{}:{}", self.prefix, self.encode_payload())
    }

    /// Decode address string: `jio:qp...` -> Address
    pub fn decode(addr_str: &str) -> Result<Self, AddressError> {
        let addr_str = addr_str.trim();
        let parts: Vec<&str> = addr_str.split(':').collect();
        if parts.len() != 2 {
            return Err(AddressError::InvalidFormat);
        }
        let prefix = Prefix::from_str(parts[0])?;
        Self::decode_payload(prefix, parts[1])
    }
}

impl FromStr for Address {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode(s)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.prefix, self.encode_payload())
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
            serializer.serialize_str(&self.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_encode_and_decode_roundtrip() {
        let pubkey_bytes = vec![0x42; 32];
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, pubkey_bytes.clone());

        let encoded = addr.encode();
        assert!(encoded.starts_with("jio:"));

        let decoded = Address::decode(&encoded).expect("decoding should succeed");
        assert_eq!(decoded.prefix, Prefix::Mainnet);
        assert_eq!(decoded.version, Version::PubKey);
        assert_eq!(decoded.payload, pubkey_bytes);
    }

    #[test]
    fn test_all_network_prefixes() {
        let key = vec![0x77; 32];
        for (prefix, expected_prefix_str) in [
            (Prefix::Mainnet, "jio:"),
            (Prefix::Testnet, "jiotest:"),
            (Prefix::Simnet, "jiosim:"),
            (Prefix::Devnet, "jiodev:"),
        ] {
            let addr = Address::new(prefix, Version::PubKey, key.clone());
            let encoded = addr.encode();
            assert!(encoded.starts_with(expected_prefix_str));

            let decoded = Address::decode(&encoded).unwrap();
            assert_eq!(decoded.prefix, prefix);
            assert_eq!(decoded.version, Version::PubKey);
            assert_eq!(decoded.payload, key);
        }
    }

    #[test]
    fn test_ecdsa_and_scripthash_versions() {
        let ecdsa_key = vec![0x02; 33]; // 33-byte compressed secp256k1 key
        let addr_ecdsa = Address::new(Prefix::Mainnet, Version::PubKeyECDSA, ecdsa_key.clone());
        let encoded_ecdsa = addr_ecdsa.encode();
        let decoded_ecdsa = Address::decode(&encoded_ecdsa).unwrap();
        assert_eq!(decoded_ecdsa.version, Version::PubKeyECDSA);
        assert_eq!(decoded_ecdsa.payload, ecdsa_key);

        let script_hash = vec![0xAA; 32]; // 32-byte P2SH script hash
        let addr_p2sh = Address::new(Prefix::Testnet, Version::ScriptHash, script_hash.clone());
        let encoded_p2sh = addr_p2sh.encode();
        let decoded_p2sh = Address::decode(&encoded_p2sh).unwrap();
        assert_eq!(decoded_p2sh.version, Version::ScriptHash);
        assert_eq!(decoded_p2sh.payload, script_hash);
    }

    #[test]
    fn test_invalid_address_rejection() {
        // Missing colon delimiter
        assert_eq!(
            Address::decode("jiotestqp3d"),
            Err(AddressError::InvalidFormat)
        );

        // Short or empty payload
        assert!(Address::decode("jio:").is_err());
        assert!(Address::decode("jio:qqqqqqqq").is_err());

        // Unknown prefix
        assert!(matches!(
            Address::decode("bitcoin:qp3d"),
            Err(AddressError::UnknownPrefix(_))
        ));

        // Tampered checksum (modified last char)
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, vec![0x11; 32]);
        let mut encoded = addr.encode();
        let last_char = encoded.pop().unwrap();
        let tampered_char = if last_char == 'q' { 'p' } else { 'q' };
        encoded.push(tampered_char);
        assert_eq!(Address::decode(&encoded), Err(AddressError::BadChecksum));
    }

    #[test]
    fn test_address_serde_json() {
        let addr = Address::new(Prefix::Mainnet, Version::PubKey, vec![0x55; 32]);
        let json = serde_json::to_string(&addr).unwrap();
        let de: Address = serde_json::from_str(&json).unwrap();
        assert_eq!(addr, de);
    }
}
