use jio_addresses::Prefix;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum NetworkError {
    #[error("invalid network name '{0}'")]
    InvalidNetwork(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
}

impl NetworkType {
    pub const fn iter() -> [Self; 4] {
        [
            NetworkType::Mainnet,
            NetworkType::Testnet,
            NetworkType::Devnet,
            NetworkType::Simnet,
        ]
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            NetworkType::Mainnet => "mainnet",
            NetworkType::Testnet => "testnet",
            NetworkType::Devnet => "devnet",
            NetworkType::Simnet => "simnet",
        }
    }

    pub const fn default_p2p_port(&self) -> u16 {
        match self {
            NetworkType::Mainnet => 16111,
            NetworkType::Testnet => 16211,
            NetworkType::Devnet => 16311,
            NetworkType::Simnet => 16411,
        }
    }

    pub const fn default_rpc_port(&self) -> u16 {
        match self {
            NetworkType::Mainnet => 16110,
            NetworkType::Testnet => 16210,
            NetworkType::Devnet => 16310,
            NetworkType::Simnet => 16410,
        }
    }

    pub const fn default_wrpc_port(&self) -> u16 {
        match self {
            NetworkType::Mainnet => 17110,
            NetworkType::Testnet => 17210,
            NetworkType::Devnet => 17310,
            NetworkType::Simnet => 17410,
        }
    }

    pub fn into_prefix(self) -> Prefix {
        match self {
            NetworkType::Mainnet => Prefix::Mainnet,
            NetworkType::Testnet => Prefix::Testnet,
            NetworkType::Devnet => Prefix::Devnet,
            NetworkType::Simnet => Prefix::Simnet,
        }
    }
}

impl From<NetworkType> for Prefix {
    fn from(net: NetworkType) -> Self {
        net.into_prefix()
    }
}

impl fmt::Display for NetworkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for NetworkType {
    type Err = NetworkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" | "jio" => Ok(NetworkType::Mainnet),
            "testnet" | "jiotest" => Ok(NetworkType::Testnet),
            "devnet" | "jiodev" => Ok(NetworkType::Devnet),
            "simnet" | "jiosim" => Ok(NetworkType::Simnet),
            _ => Err(NetworkError::InvalidNetwork(s.to_string())),
        }
    }
}

impl Serialize for NetworkType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NetworkType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NetworkType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NetworkId {
    pub network_type: NetworkType,
    pub suffix: Option<u32>,
}

impl NetworkId {
    pub const fn new(network_type: NetworkType) -> Self {
        Self {
            network_type,
            suffix: None,
        }
    }

    pub const fn with_suffix(network_type: NetworkType, suffix: u32) -> Self {
        Self {
            network_type,
            suffix: Some(suffix),
        }
    }

    pub fn is_mainnet(&self) -> bool {
        self.network_type == NetworkType::Mainnet
    }

    pub fn to_prefixed(&self) -> String {
        format!("{self}")
    }

    pub fn default_p2p_port(&self) -> u16 {
        self.network_type.default_p2p_port()
    }

    pub fn default_rpc_port(&self) -> u16 {
        self.network_type.default_rpc_port()
    }
}

impl From<NetworkId> for Prefix {
    fn from(net: NetworkId) -> Self {
        net.network_type.into_prefix()
    }
}

impl From<NetworkType> for NetworkId {
    fn from(network_type: NetworkType) -> Self {
        Self::new(network_type)
    }
}


impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(suffix) = self.suffix {
            write!(f, "{}-{}", self.network_type, suffix)
        } else {
            write!(f, "{}", self.network_type)
        }
    }
}

impl FromStr for NetworkId {
    type Err = NetworkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.is_empty() {
            return Err(NetworkError::InvalidNetwork(s.to_string()));
        }
        let network_type = NetworkType::from_str(parts[0])?;
        let suffix = if parts.len() > 1 {
            parts[1].parse::<u32>().ok()
        } else {
            None
        };
        Ok(Self {
            network_type,
            suffix,
        })
    }
}

impl Serialize for NetworkId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for NetworkId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NetworkId::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_roundtrip() {
        let net = NetworkType::Mainnet;
        assert_eq!(net.default_p2p_port(), 16111);
        let id = NetworkId::with_suffix(NetworkType::Testnet, 11);
        assert_eq!(id.to_string(), "testnet-11");
        let parsed: NetworkId = "testnet-11".parse().unwrap();
        assert_eq!(id, parsed);
    }
}
