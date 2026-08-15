use jio_consensus_core::tx::{ScriptPublicKey, TransactionOutpoint};
use jio_consensus_core::utxo::UtxoEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type UtxoSetByScriptPublicKey = HashMap<ScriptPublicKey, HashMap<TransactionOutpoint, UtxoEntry>>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct IndexedUtxos {
    pub utxos: UtxoSetByScriptPublicKey,
}

impl IndexedUtxos {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
        }
    }

    pub fn insert(&mut self, spk: ScriptPublicKey, outpoint: TransactionOutpoint, entry: UtxoEntry) {
        self.utxos.entry(spk).or_default().insert(outpoint, entry);
    }

    pub fn remove(&mut self, spk: &ScriptPublicKey, outpoint: &TransactionOutpoint) -> Option<UtxoEntry> {
        if let Some(entries) = self.utxos.get_mut(spk) {
            let res = entries.remove(outpoint);
            if entries.is_empty() {
                self.utxos.remove(spk);
            }
            res
        } else {
            None
        }
    }

    pub fn get_by_spk(&self, spk: &ScriptPublicKey) -> Option<&HashMap<TransactionOutpoint, UtxoEntry>> {
        self.utxos.get(spk)
    }

    pub fn total_balance_by_spk(&self, spk: &ScriptPublicKey) -> u64 {
        self.utxos
            .get(spk)
            .map(|entries| entries.values().map(|e| e.amount).sum())
            .unwrap_or(0)
    }
}
