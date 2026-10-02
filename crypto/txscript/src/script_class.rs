//! Script classification types.

use jio_addresses::{Address, Prefix, Version};
use super::opcodes::macros::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub enum ScriptClass {
    /// Pay-to-pubkey: <32-byte Schnorr pubkey> OP_CHECKSIG
    PubKey,
    /// Pay-to-ECDSA-pubkey: <33-byte compressed ECDSA pubkey> OP_CHECKSIG
    PubKeyECDSA,
    /// Pay-to-script-hash: OP_HASH256 <32-byte script hash> OP_EQUAL
    ScriptHash,
    /// Non-standard / unclassified script
    NonStandard,
}

impl ScriptClass {
    /// Classifies an output script (ScriptPublicKey bytes).
    pub fn from_script(script: &[u8]) -> Self {
        if script.len() == 34 && script[0] == 0x20 && script[33] == OP_CHECKSIG {
            Self::PubKey
        } else if script.len() == 35 && script[0] == 0x21 && script[34] == OP_CHECKSIG {
            Self::PubKeyECDSA
        } else if script.len() == 35 && script[0] == OP_HASH256 && script[1] == 0x20 && script[34] == OP_EQUAL {
            Self::ScriptHash
        } else {
            Self::NonStandard
        }
    }

    /// Extracts the destination address from standard scripts if identifiable.
    pub fn extract_address(&self, script: &[u8], prefix: Prefix) -> Option<Address> {
        match self {
            Self::PubKey => {
                if script.len() == 34 {
                    Some(Address::new(prefix, Version::PubKey, &script[1..33]))
                } else {
                    None
                }
            }
            Self::PubKeyECDSA => {
                if script.len() == 35 {
                    Some(Address::new(prefix, Version::PubKeyECDSA, &script[1..34]))
                } else {
                    None
                }
            }
            Self::ScriptHash => {
                if script.len() == 35 {
                    Some(Address::new(prefix, Version::ScriptHash, &script[2..34]))
                } else {
                    None
                }
            }
            Self::NonStandard => None,
        }
    }
}
