// utxo/utxo_collection.rs
//
// In-memory collection of UTXOs keyed by TransactionOutpoint.

use crate::tx::{TransactionOutpoint, UtxoEntry};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type UtxoCollection = HashMap<TransactionOutpoint, UtxoEntry>;

#[derive(Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UtxoEntryWithOutpoint {
    pub outpoint: TransactionOutpoint,
    pub entry: UtxoEntry,
}
