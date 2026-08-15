use crate::processes::transaction_validator::errors::TxRuleError;
use jio_consensus_core::constants::MAX_SOMPI;
use jio_consensus_core::hashing::tx::tx_id;
use jio_consensus_core::tx::Transaction;
use std::collections::HashSet;

const MAX_SCRIPT_PUBLIC_KEY_SIZE: usize = 1024;

pub fn check_tx_in_isolation(tx: &Transaction) -> Result<(), TxRuleError> {
    let id = tx_id(tx);

    if tx.is_coinbase() {
        if tx.outputs.is_empty() {
            return Err(TxRuleError::NoOutputs(id));
        }
    } else {
        if tx.inputs.is_empty() {
            return Err(TxRuleError::NoInputs(id));
        }
        if tx.outputs.is_empty() {
            return Err(TxRuleError::NoOutputs(id));
        }

        let mut seen = HashSet::with_capacity(tx.inputs.len());
        for input in &tx.inputs {
            if !seen.insert(input.previous_outpoint) {
                return Err(TxRuleError::DuplicateInputs(id));
            }
        }
    }

    let mut total_out: u64 = 0;
    for output in &tx.outputs {
        if output.value > MAX_SOMPI {
            return Err(TxRuleError::ValueExceedsMaxSompi(output.value));
        }
        total_out = total_out
            .checked_add(output.value)
            .ok_or(TxRuleError::ValueExceedsMaxSompi(output.value))?;
        if total_out > MAX_SOMPI {
            return Err(TxRuleError::ValueExceedsMaxSompi(total_out));
        }
        if output.script_public_key.script().len() > MAX_SCRIPT_PUBLIC_KEY_SIZE {
            return Err(TxRuleError::ScriptPublicKeyTooLarge(
                output.script_public_key.script().len(),
                MAX_SCRIPT_PUBLIC_KEY_SIZE,
            ));
        }
    }

    Ok(())
}
