use crate::mempool::config::MempoolConfig;
use jio_consensus_core::tx::Transaction;
use jio_mining_errors::{MiningError, MiningResult};

pub fn check_transaction_standard(tx: &Transaction, config: &MempoolConfig) -> MiningResult<()> {
    if tx.inputs.is_empty() {
        return Err(MiningError::TxRejected("transaction has no inputs".to_string()));
    }
    if tx.outputs.is_empty() {
        return Err(MiningError::TxRejected("transaction has no outputs".to_string()));
    }

    let mass = tx.calc_mass();
    if mass > config.maximum_block_mass {
        return Err(MiningError::MassExceeded(mass, config.maximum_block_mass));
    }

    for (idx, output) in tx.outputs.iter().enumerate() {
        if output.value == 0 {
            return Err(MiningError::TxRejected(format!("output {} value is 0 (dust)", idx)));
        }
        if output.script_public_key.script().len() > 10_000 {
            return Err(MiningError::TxRejected(format!(
                "output {} script length {} exceeds 10,000 bytes",
                idx,
                output.script_public_key.script().len()
            )));
        }
    }

    Ok(())
}
