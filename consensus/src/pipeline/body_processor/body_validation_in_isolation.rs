use crate::processes::transaction_validator::tx_validation_in_isolation::check_tx_in_isolation;
use jio_consensus_core::block::Block;
use jio_consensus_core::config::params::MAINNET_PARAMS;
use jio_consensus_core::errors::block::BlockRuleError;

const MAX_BLOCK_TRANSACTIONS: usize = 10_000;

pub fn validate_body_in_isolation(block: &Block) -> Result<(), BlockRuleError> {
    if block.transactions.is_empty() {
        return Err(BlockRuleError::InvalidTx("block has no transactions".to_string()));
    }

    if block.transactions.len() > MAX_BLOCK_TRANSACTIONS {
        return Err(BlockRuleError::InvalidTx("too many transactions in block".to_string()));
    }

    if !block.transactions[0].is_coinbase() {
        return Err(BlockRuleError::InvalidTx("first transaction must be coinbase".to_string()));
    }

    for (i, tx) in block.transactions.iter().enumerate().skip(1) {
        if tx.is_coinbase() {
            return Err(BlockRuleError::InvalidTx(format!(
                "transaction at index {i} cannot be coinbase"
            )));
        }
        check_tx_in_isolation(tx).map_err(|e| BlockRuleError::InvalidTx(e.to_string()))?;
    }

    let total_mass: u64 = block.transactions.iter().map(|tx| tx.mass).sum();
    if total_mass > MAINNET_PARAMS.max_block_mass {
        return Err(BlockRuleError::ExceedsMassLimit(total_mass, MAINNET_PARAMS.max_block_mass));
    }

    Ok(())
}
