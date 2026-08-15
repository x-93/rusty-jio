use crate::model::stores::utxo_set::UtxoSetStore;
use jio_consensus_core::errors::tx::TxRuleError;
use jio_consensus_core::hashing::tx::tx_id;
use jio_consensus_core::tx::{Transaction, TransactionOutpoint};
use jio_consensus_core::utxo::{UtxoDiff, UtxoEntry, UtxoView};

pub fn validate_and_apply_tx_utxo(
    tx: &Transaction,
    utxo_view: &UtxoSetStore,
    current_daa_score: u64,
    coinbase_maturity: u64,
    accumulated_diff: &mut UtxoDiff,
) -> Result<u64, TxRuleError> {
    let id = tx_id(tx);
    let mut total_in: u64 = 0;

    if !tx.is_coinbase() {
        for input in &tx.inputs {
            let outpoint = input.previous_outpoint;

            if accumulated_diff.to_remove.contains_key(&outpoint) {
                return Err(TxRuleError::ScriptFailed("double spend detected".to_string()));
            }

            let entry = if let Some(entry) = accumulated_diff.to_add.get(&outpoint) {
                entry.clone()
            } else if let Some(entry) = utxo_view.get(&outpoint) {
                entry
            } else {
                return Err(TxRuleError::ScriptFailed("UTXO entry not found".to_string()));
            };

            if entry.is_coinbase && entry.block_daa_score + coinbase_maturity > current_daa_score {
                return Err(TxRuleError::ScriptFailed(
                    "coinbase maturity not reached".to_string(),
                ));
            }

            total_in = total_in
                .checked_add(entry.amount)
                .ok_or(TxRuleError::ValueExceedsMaxSompi(entry.amount))?;

            accumulated_diff.to_remove.insert(outpoint, entry);
        }
    }

    let total_out = tx.total_out_value();
    let fee = if tx.is_coinbase() {
        0
    } else {
        if total_in < total_out {
            return Err(TxRuleError::NegativeFee((total_in as i64) - (total_out as i64)));
        }
        total_in - total_out
    };

    for (i, output) in tx.outputs.iter().enumerate() {
        let outpoint = TransactionOutpoint::new(id, i as u32);
        let entry = UtxoEntry::new(
            output.value,
            output.script_public_key.clone(),
            current_daa_score,
            tx.is_coinbase(),
        );
        accumulated_diff.to_add.insert(outpoint, entry);
    }

    Ok(fee)
}
