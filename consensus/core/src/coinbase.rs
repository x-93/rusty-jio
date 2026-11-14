// coinbase.rs
//
// Miner data and coinbase payload structures for rusty-jio.

use borsh::{BorshDeserialize, BorshSerialize};
use jio_txscript::ScriptPublicKey;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct MinerData {
    pub script_public_key: ScriptPublicKey,
    pub extra_data: Vec<u8>,
}

impl MinerData {
    pub fn new(script_public_key: ScriptPublicKey, extra_data: Vec<u8>) -> Self {
        Self {
            script_public_key,
            extra_data,
        }
    }
}
