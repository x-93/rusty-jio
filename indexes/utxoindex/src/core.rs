use jio_consensus_core::tx::{ScriptPublicKey, TransactionOutpoint};
use jio_consensus_core::utxo::UtxoEntry;
use std::collections::HashMap;

pub trait UtxoIndexReader: Send + Sync {
    fn get_utxos_by_script_public_key(
        &self,
        spk: &ScriptPublicKey,
    ) -> Option<HashMap<TransactionOutpoint, UtxoEntry>>;

    fn get_circulating_supply(&self) -> u64;
}
