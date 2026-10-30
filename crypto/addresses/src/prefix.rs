use serde::{Deserialize, Serialize};
use std::fmt;

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

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "jio" => Ok(Prefix::Mainnet),
            "jiotest" => Ok(Prefix::Testnet),
            "jiosim" => Ok(Prefix::Simnet),
            "jiodev" => Ok(Prefix::Devnet),
            _ => Err(format!("Unknown address prefix: '{}'", s)),
        }
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
