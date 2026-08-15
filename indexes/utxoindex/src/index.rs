use crate::core::UtxoIndexReader;
use crate::stores::UtxoIndexStores;
use crate::update::UtxoIndexUpdater;
use jio_consensus_core::tx::{ScriptPublicKey, TransactionOutpoint};
use jio_consensus_core::utxo::{UtxoDiff, UtxoEntry};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct UtxoIndex {
    stores: UtxoIndexStores,
}

impl UtxoIndex {
    pub fn new() -> Self {
        Self {
            stores: UtxoIndexStores::new(),
        }
    }

    pub fn update(&self, diff: &UtxoDiff) {
        UtxoIndexUpdater::update(&self.stores, diff);
    }
}

impl UtxoIndexReader for UtxoIndex {
    fn get_utxos_by_script_public_key(
        &self,
        spk: &ScriptPublicKey,
    ) -> Option<HashMap<TransactionOutpoint, UtxoEntry>> {
        self.stores.indexed_utxos.read().get_by_spk(spk).cloned()
    }

    fn get_circulating_supply(&self) -> u64 {
        self.stores.circulating_supply.read().total_amount
    }
}
