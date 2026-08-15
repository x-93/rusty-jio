pub mod bech32;

pub use bech32::{convert_bits, Bech32Error};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum AddressError {
    #[error("bech32 error: {0}")]
    Bech32(#[from] Bech32Error),
    #[error("invalid address version: {0}")]
    InvalidVersion(u8),
    #[error("invalid address payload length: expected {expected}, got {actual}")]
    InvalidPayloadLength { expected: usize, actual: usize },
    #[error("invalid prefix: {0}")]
    InvalidPrefix(String),
    #[error("unsupported script class: {0}")]
    UnsupportedScriptClass(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AddressVersion {
    PubKey = 0,
    PubKeyECDSA = 1,
    ScriptHash = 8,
}

impl TryFrom<u8> for AddressVersion {
    type Error = AddressError;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(AddressVersion::PubKey),
            1 => Ok(AddressVersion::PubKeyECDSA),
            8 => Ok(AddressVersion::ScriptHash),
            other => Err(AddressError::InvalidVersion(other)),
        }
    }
}

impl From<AddressVersion> for u8 {
    fn from(v: AddressVersion) -> Self {
        v as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Prefix {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
    Custom(String),
}

impl Prefix {
    pub fn as_str(&self) -> &str {
        match self {
            Prefix::Mainnet => "jio",
            Prefix::Testnet => "jiotest",
            Prefix::Devnet => "jiodev",
            Prefix::Simnet => "jiosim",
            Prefix::Custom(s) => s.as_str(),
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
        match s {
            "jio" => Ok(Prefix::Mainnet),
            "jiotest" => Ok(Prefix::Testnet),
            "jiodev" => Ok(Prefix::Devnet),
            "jiosim" => Ok(Prefix::Simnet),
            other => Ok(Prefix::Custom(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address {
    pub prefix: Prefix,
    pub version: AddressVersion,
    pub payload: Vec<u8>,
}

impl Address {
    pub fn new(prefix: Prefix, version: AddressVersion, payload: Vec<u8>) -> Self {
        Self {
            prefix,
            version,
            payload,
        }
    }

    pub fn to_string_with_prefix(&self) -> String {
        let mut data = Vec::with_capacity(1 + self.payload.len());
        data.push(self.version.into());
        data.extend_from_slice(&self.payload);

        let data_5bit = convert_bits(&data, 8, 5, true).expect("valid 8 to 5 bit conversion");
        bech32::encode(self.prefix.as_str(), &data_5bit)
    }

    pub fn to_script_pub_key(&self) -> jio_txscript::ScriptPublicKey {
        pay_to_address_script(self)
    }

    pub fn from_script_pub_key(
        spk: &jio_txscript::ScriptPublicKey,
        prefix: Prefix,
    ) -> Result<Self, AddressError> {
        extract_script_pub_key_address(spk, prefix)
    }
}

pub fn pay_to_address_script(address: &Address) -> jio_txscript::ScriptPublicKey {
    match address.version {
        AddressVersion::PubKey => {
            if address.payload.len() == 32 {
                let mut pk = [0u8; 32];
                pk.copy_from_slice(&address.payload);
                jio_txscript::standard::pay_to_pubkey_script(&pk)
            } else {
                jio_txscript::ScriptPublicKey::new(0, vec![])
            }
        }
        AddressVersion::PubKeyECDSA => {
            if address.payload.len() == 33 {
                let mut pk = [0u8; 33];
                pk.copy_from_slice(&address.payload);
                jio_txscript::standard::pay_to_pubkey_ecdsa_script(&pk)
            } else {
                jio_txscript::ScriptPublicKey::new(0, vec![])
            }
        }
        AddressVersion::ScriptHash => {
            if address.payload.len() == 32 {
                let mut sh = [0u8; 32];
                sh.copy_from_slice(&address.payload);
                jio_txscript::standard::pay_to_script_hash_script(&sh)
            } else {
                jio_txscript::ScriptPublicKey::new(0, vec![])
            }
        }
    }
}

pub fn extract_script_pub_key_address(
    spk: &jio_txscript::ScriptPublicKey,
    prefix: Prefix,
) -> Result<Address, AddressError> {
    let script = spk.script();
    let class = jio_txscript::script_class::classify_script(script);
    match class {
        jio_txscript::script_class::ScriptClass::PubKey => {
            if script.len() == 34 {
                let payload = script[1..33].to_vec();
                Ok(Address::new(prefix, AddressVersion::PubKey, payload))
            } else {
                Err(AddressError::InvalidPayloadLength {
                    expected: 32,
                    actual: script.len().saturating_sub(2),
                })
            }
        }
        jio_txscript::script_class::ScriptClass::PubKeyECDSA => {
            if script.len() == 35 {
                let payload = script[1..34].to_vec();
                Ok(Address::new(prefix, AddressVersion::PubKeyECDSA, payload))
            } else {
                Err(AddressError::InvalidPayloadLength {
                    expected: 33,
                    actual: script.len().saturating_sub(2),
                })
            }
        }
        jio_txscript::script_class::ScriptClass::ScriptHash => {
            if script.len() == 35 {
                let payload = script[2..34].to_vec();
                Ok(Address::new(prefix, AddressVersion::ScriptHash, payload))
            } else {
                Err(AddressError::InvalidPayloadLength {
                    expected: 32,
                    actual: script.len().saturating_sub(3),
                })
            }
        }
        _ => Err(AddressError::UnsupportedScriptClass(format!("{:?}", class))),
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_prefix())
    }
}

impl FromStr for Address {
    type Err = AddressError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hrp, data_5bit) = bech32::decode(s)?;
        let prefix = Prefix::from_str(&hrp)?;
        let data_8bit = convert_bits(&data_5bit, 5, 8, false)?;
        if data_8bit.is_empty() {
            return Err(AddressError::InvalidPayloadLength {
                expected: 32,
                actual: 0,
            });
        }
        let version = AddressVersion::try_from(data_8bit[0])?;
        let payload = data_8bit[1..].to_vec();

        match version {
            AddressVersion::PubKey | AddressVersion::ScriptHash => {
                if payload.len() != 32 {
                    return Err(AddressError::InvalidPayloadLength {
                        expected: 32,
                        actual: payload.len(),
                    });
                }
            }
            AddressVersion::PubKeyECDSA => {
                if payload.len() != 33 {
                    return Err(AddressError::InvalidPayloadLength {
                        expected: 33,
                        actual: payload.len(),
                    });
                }
            }
        }

        Ok(Address {
            prefix,
            version,
            payload,
        })
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string_with_prefix())
        } else {
            let mut bytes = Vec::with_capacity(1 + self.payload.len());
            bytes.push(self.version.into());
            bytes.extend_from_slice(&self.payload);
            serializer.serialize_bytes(&bytes)
        }
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_roundtrip() {
        let pk = [42u8; 32];
        let addr = Address::new(Prefix::Mainnet, AddressVersion::PubKey, pk.to_vec());
        let addr_str = addr.to_string();
        assert!(addr_str.starts_with("jio:"));

        let parsed: Address = addr_str.parse().unwrap();
        assert_eq!(addr, parsed);
    }

    #[test]
    fn test_testnet_address() {
        let pk = [99u8; 32];
        let addr = Address::new(Prefix::Testnet, AddressVersion::PubKey, pk.to_vec());
        let addr_str = addr.to_string();
        assert!(addr_str.starts_with("jiotest:"));

        let parsed: Address = addr_str.parse().unwrap();
        assert_eq!(addr.prefix, Prefix::Testnet);
        assert_eq!(addr, parsed);
    }

    #[test]
    fn test_address_script_pubkey_roundtrip() {
        let pk = [77u8; 32];
        let addr = Address::new(Prefix::Mainnet, AddressVersion::PubKey, pk.to_vec());
        let spk = addr.to_script_pub_key();
        let extracted = Address::from_script_pub_key(&spk, Prefix::Mainnet).unwrap();
        assert_eq!(addr, extracted);

        let pke = [88u8; 33];
        let addr_ecdsa = Address::new(Prefix::Testnet, AddressVersion::PubKeyECDSA, pke.to_vec());
        let spk_ecdsa = addr_ecdsa.to_script_pub_key();
        let extracted_ecdsa = Address::from_script_pub_key(&spk_ecdsa, Prefix::Testnet).unwrap();
        assert_eq!(addr_ecdsa, extracted_ecdsa);

        let sh = [99u8; 32];
        let addr_sh = Address::new(Prefix::Devnet, AddressVersion::ScriptHash, sh.to_vec());
        let spk_sh = addr_sh.to_script_pub_key();
        let extracted_sh = Address::from_script_pub_key(&spk_sh, Prefix::Devnet).unwrap();
        assert_eq!(addr_sh, extracted_sh);
    }
}
