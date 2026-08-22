pub mod utxo_collection;
pub mod utxo_diff;
pub mod utxo_error;
pub mod utxo_view;

pub use utxo_collection::*;
pub use utxo_diff::*;
pub use utxo_error::*;
pub use utxo_view::*;

use crate::tx::ScriptPublicKey;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Represents an unspent transaction output (UTXO) with consensus metadata.
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UtxoEntry {
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
    pub block_daa_score: u64,
    pub is_coinbase: bool,
}

impl UtxoEntry {
    pub const fn new(
        amount: u64,
        script_public_key: ScriptPublicKey,
        block_daa_score: u64,
        is_coinbase: bool,
    ) -> Self {
        Self {
            amount,
            script_public_key,
            block_daa_score,
            is_coinbase,
        }
    }
}
