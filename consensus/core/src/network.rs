use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

/// The broad network classification for Jio consensus.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
}

impl NetworkType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Devnet => "devnet",
            Self::Simnet => "simnet",
        }
    }

    pub const fn default_p2p_port(&self) -> u16 {
        match self {
            Self::Mainnet => 29111,
            Self::Testnet => 29211,
            Self::Devnet => 29311,
            Self::Simnet => 29411,
        }
    }

    pub const fn default_rpc_port(&self) -> u16 {
        match self {
            Self::Mainnet => 29110,
            Self::Testnet => 29210,
            Self::Devnet => 29310,
            Self::Simnet => 29410,
        }
    }

    pub const fn default_wrpc_borsh_port(&self) -> u16 {
        match self {
            Self::Mainnet => 29112,
            Self::Testnet => 29212,
            Self::Devnet => 29312,
            Self::Simnet => 29412,
        }
    }

    pub const fn default_wrpc_json_port(&self) -> u16 {
        match self {
            Self::Mainnet => 29113,
            Self::Testnet => 29213,
            Self::Devnet => 29313,
            Self::Simnet => 29413,
        }
    }
}

impl Display for NetworkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NetworkType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "devnet" => Ok(Self::Devnet),
            "simnet" => Ok(Self::Simnet),
            _ => Err("unknown network type"),
        }
    }
}

/// A specific network identifier including optional testnet suffix (e.g. `testnet-11`).
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
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

    pub const fn is_mainnet(&self) -> bool {
        matches!(self.network_type, NetworkType::Mainnet)
    }
}

impl From<NetworkType> for NetworkId {
    fn from(net_type: NetworkType) -> Self {
        Self::new(net_type)
    }
}

impl Display for NetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.suffix {
            Some(suffix) => write!(f, "{}-{}", self.network_type, suffix),
            None => write!(f, "{}", self.network_type),
        }
    }
}

impl FromStr for NetworkId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, suffix) = match s.split_once('-') {
            Some((name, suffix_str)) => {
                let suffix = suffix_str
                    .parse::<u32>()
                    .map_err(|_| "invalid network suffix")?;
                (name, Some(suffix))
            }
            None => (s, None),
        };
        let network_type = NetworkType::from_str(name)?;
        Ok(Self {
            network_type,
            suffix,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_id_parsing() {
        let mainnet: NetworkId = "mainnet".parse().unwrap();
        assert_eq!(mainnet, NetworkId::new(NetworkType::Mainnet));
        assert!(mainnet.is_mainnet());

        let testnet11: NetworkId = "testnet-11".parse().unwrap();
        assert_eq!(testnet11, NetworkId::with_suffix(NetworkType::Testnet, 11));
        assert_eq!(testnet11.to_string(), "testnet-11");
    }
}
