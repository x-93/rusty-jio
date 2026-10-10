use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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
}

impl From<NetworkType> for NetworkId {
    fn from(net_type: NetworkType) -> Self {
        Self::new(net_type)
    }
}

impl Display for NetworkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (self.network_type, self.suffix) {
            (NetworkType::Mainnet, _) => write!(f, "mainnet"),
            (NetworkType::Testnet, None) => write!(f, "testnet"),
            (NetworkType::Testnet, Some(s)) => write!(f, "testnet-{}", s),
            (NetworkType::Devnet, _) => write!(f, "devnet"),
            (NetworkType::Simnet, _) => write!(f, "simnet"),
        }
    }
}