use super::errors::{VirtualResult, VirtualStateError};
use crate::processes::transaction_validator::validate_populated_transaction_and_calc_fee;
use consensus_core::tx::{Transaction, TransactionOutpoint, UtxoEntry};
use consensus_core::utxo::utxo_collection::UtxoCollection;
use consensus_core::utxo::utxo_diff::UtxoDiff;

/// Processes and validates transactions in a block against the current UTXO state,
/// applying mutations and constructing a deterministic `UtxoDiff`.
pub fn process_block_transactions_and_build_diff(
    transactions: &[Transaction],
    utxo_set: &mut UtxoCollection,
    current_daa_score: u64,
    coinbase_maturity: u64,
) -> VirtualResult<(UtxoDiff, Vec<u64>)> {
    let mut diff = UtxoDiff::default();
    let mut calculated_fees = Vec::with_capacity(transactions.len());

    for tx in transactions {
        let tx_id = tx.id();

        if tx.is_coinbase() {
            // Coinbase transactions produce outputs without spending any inputs
            for (idx, output) in tx.outputs.iter().enumerate() {
                let outpoint = TransactionOutpoint::new(tx_id, idx as u32);
                let entry = UtxoEntry::new(
                    output.value,
                    output.script_public_key.clone(),
                    current_daa_score,
                    true,
                );
                utxo_set.insert(outpoint, entry.clone());
                diff.add_entry(outpoint, entry);
            }
            calculated_fees.push(0);
        } else {
            // 1. Validate populated inputs and calculate transaction fee
            let fee = validate_populated_transaction_and_calc_fee(
                tx,
                utxo_set,
                current_daa_score,
                coinbase_maturity,
            )
            .map_err(VirtualStateError::Tx)?;
            calculated_fees.push(fee);

            // 2. Remove spent inputs from UTXO state
            for input in &tx.inputs {
                if let Some(spent_utxo) = utxo_set.remove(&input.previous_outpoint) {
                    diff.remove_entry(input.previous_outpoint, spent_utxo);
                }
            }

            // 3. Add newly created outputs to UTXO state
            for (idx, output) in tx.outputs.iter().enumerate() {
                let outpoint = TransactionOutpoint::new(tx_id, idx as u32);
                let entry = UtxoEntry::new(
                    output.value,
                    output.script_public_key.clone(),
                    current_daa_score,
                    false,
                );
                utxo_set.insert(outpoint, entry.clone());
                diff.add_entry(outpoint, entry);
            }
        }
    }

    Ok((diff, calculated_fees))
}
