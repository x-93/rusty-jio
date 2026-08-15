use crate::stores::UtxoIndexStores;
use jio_consensus_core::utxo::UtxoDiff;

pub struct UtxoIndexUpdater;

impl UtxoIndexUpdater {
    pub fn update(stores: &UtxoIndexStores, utxo_diff: &UtxoDiff) {
        let mut index = stores.indexed_utxos.write();
        let mut supply = stores.circulating_supply.write();

        for (outpoint, entry) in &utxo_diff.to_remove {
            index.remove(&entry.script_public_key, outpoint);
            supply.total_amount = supply.total_amount.saturating_sub(entry.amount);
        }

        for (outpoint, entry) in &utxo_diff.to_add {
            index.insert(entry.script_public_key.clone(), *outpoint, entry.clone());
            supply.total_amount = supply.total_amount.saturating_add(entry.amount);
        }
    }
}
