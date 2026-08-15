use jio_consensus_core::tx::{ScriptPublicKey, TransactionOutpoint};
use jio_consensus_core::utxo::UtxoEntry;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CirculatingSupply {
    pub total_amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoIndexEntry {
    pub spk: ScriptPublicKey,
    pub outpoint: TransactionOutpoint,
    pub entry: UtxoEntry,
}
