use crate::processes::transaction_validator::errors::TxRuleError;
use jio_consensus_core::tx::Transaction;

pub fn check_tx_finalized(
    tx: &Transaction,
    current_daa_score: u64,
    past_median_time: u64,
) -> Result<(), TxRuleError> {
    if tx.lock_time == 0 {
        return Ok(());
    }

    let threshold = if tx.lock_time < 500_000_000 {
        current_daa_score
    } else {
        past_median_time
    };

    if tx.lock_time > threshold {
        return Err(TxRuleError::NotFinalized(tx.lock_time));
    }

    Ok(())
}
