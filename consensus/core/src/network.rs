use borsh::{BorshDeserialize, BorshSerialize};
use jio_addresses::Prefix;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
}

impl NetworkType {
    pub fn default_prefix(&self) -> Prefix {
        match self {
            Self::Mainnet => Prefix::Mainnet,
            Self::Testnet => Prefix::Testnet,
            Self::Devnet => Prefix::Devnet,
            Self::Simnet => Prefix::Simnet,
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Mainnet, Self::Testnet, Self::Devnet, Self::Simnet].into_iter()
    }
}

impl Display for NetworkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Devnet => write!(f, "devnet"),
            Self::Simnet => write!(f, "simnet"),
        }
    }
}

impl FromStr for NetworkType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "devnet" => Ok(Self::Devnet),
            "simnet" => Ok(Self::Simnet),
            _ => Err(format!("unknown network type: {s}")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct NetworkId {
    pub network_type: NetworkType,
    pub suffix: Option<u32>,
}

impl NetworkId {
    pub const fn new(network_type: NetworkType) -> Self {
        Self { network_type, suffix: None }
    }

    pub const fn with_suffix(network_type: NetworkType, suffix: u32) -> Self {
        Self { network_type, suffix: Some(suffix) }
    }

    pub fn to_prefixed(&self) -> Prefix {
        self.network_type.default_prefix()
    }
}

impl Display for NetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.suffix {
            Some(suffix) => write!(f, "{}-{}", self.network_type, suffix),
            None => write!(f, "{}", self.network_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_id_display() {
        let net = NetworkId::with_suffix(NetworkType::Testnet, 11);
        assert_eq!(net.to_string(), "testnet-11");
        assert_eq!(net.to_prefixed(), Prefix::Testnet);
    }
}
