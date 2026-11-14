use crate::processes::transaction_validator::{
    validate_block_transactions_not_utxo_related, validate_tx_in_isolation,
};
use consensus_core::block::Block;
use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::mass::TxMassCalculator;

/// Validates block body transactions in isolation without contextual database queries.
pub fn validate_body_in_isolation(
    block: &Block,
    mass_calculator: &TxMassCalculator,
    max_block_mass: u64,
    max_coinbase_payload_size: usize,
) -> BlockProcessResult<()> {
    // Genesis block is exempt from transaction requirements if empty
    if block.header.direct_parents().is_empty() && block.transactions.is_empty() {
        return Ok(());
    }

    // 1. Validate block-level non-UTXO constraints (coinbase position, unique IDs, block mass)
    validate_block_transactions_not_utxo_related(
        &block.transactions,
        mass_calculator,
        max_block_mass,
    )
    .map_err(|_| RuleError::ExceedsMaxBlockMass(0, max_block_mass))?;

    // 2. Validate each individual transaction in isolation
    for tx in block.transactions.iter() {
        validate_tx_in_isolation(
            tx,
            mass_calculator,
            max_block_mass,
            max_coinbase_payload_size,
        )
        .map_err(|_| RuleError::KnownInvalid(block.header.hash))?;
    }

    Ok(())
}
