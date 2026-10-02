//! Network classification, naming, and default port allocations.

use core::fmt;
use core::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub enum NetworkType {
    #[default]
    Mainnet,
    Testnet,
    Simnet,
    Devnet,
}

impl NetworkType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
            Self::Simnet => "simnet",
            Self::Devnet => "devnet",
        }
    }

    pub fn default_p2p_port(&self) -> u16 {
        match self {
            Self::Mainnet => 16111,
            Self::Testnet => 16211,
            Self::Simnet => 16511,
            Self::Devnet => 16611,
        }
    }

    pub fn default_rpc_port(&self) -> u16 {
        match self {
            Self::Mainnet => 16110,
            Self::Testnet => 16210,
            Self::Simnet => 16510,
            Self::Devnet => 16610,
        }
    }

    pub fn default_wrpc_borsh_port(&self) -> u16 {
        match self {
            Self::Mainnet => 17110,
            Self::Testnet => 17210,
            Self::Simnet => 17510,
            Self::Devnet => 17610,
        }
    }

    pub fn default_wrpc_json_port(&self) -> u16 {
        match self {
            Self::Mainnet => 18110,
            Self::Testnet => 18210,
            Self::Simnet => 18510,
            Self::Devnet => 18610,
        }
    }
}

impl fmt::Display for NetworkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for NetworkType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            "simnet" => Ok(Self::Simnet),
            "devnet" => Ok(Self::Devnet),
            other => Err(format!("Unknown network type: {}", other)),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
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

impl From<NetworkType> for NetworkId {
    fn from(network_type: NetworkType) -> Self {
        Self::new(network_type)
    }
}
