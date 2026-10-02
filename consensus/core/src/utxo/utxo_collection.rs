//! In-memory UTXO hash-map collection.

use std::collections::HashMap;
use super::super::tx::{TransactionOutpoint, UtxoEntry};

pub type UtxoCollection = HashMap<TransactionOutpoint, UtxoEntry>;
