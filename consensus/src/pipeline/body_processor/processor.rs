use super::{
    body_validation_in_context::validate_body_in_context,
    body_validation_in_isolation::validate_body_in_isolation,
};
use crate::model::stores::statuses::StatusesStore;
use consensus_core::block::Block;
use consensus_core::blockstatus::BlockStatus;
use consensus_core::errors::block::{BlockProcessResult, RuleError};
use consensus_core::mass::TxMassCalculator;
use jio_database::prelude::DbWriter;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct BodyProcessor<TStatusesStore: StatusesStore> {
    statuses_store: Arc<RwLock<TStatusesStore>>,
    mass_calculator: TxMassCalculator,
    max_block_mass: u64,
    max_coinbase_payload_size: usize,
}

impl<TStatusesStore: StatusesStore> BodyProcessor<TStatusesStore> {
    pub fn new(
        statuses_store: Arc<RwLock<TStatusesStore>>,
        mass_calculator: TxMassCalculator,
        max_block_mass: u64,
        max_coinbase_payload_size: usize,
    ) -> Self {
        Self {
            statuses_store,
            mass_calculator,
            max_block_mass,
            max_coinbase_payload_size,
        }
    }

    /// Fully validates and processes a block's body (transactions and Merkle commitments).
    pub fn process_body(
        &self,
        writer: &mut impl DbWriter,
        block: &Block,
    ) -> BlockProcessResult<()> {
        let block_hash = block.header.hash;

        // 1. Isolation validation
        validate_body_in_isolation(
            block,
            &self.mass_calculator,
            self.max_block_mass,
            self.max_coinbase_payload_size,
        )?;

        // 2. Contextual validation (Merkle root check)
        validate_body_in_context(block)?;

        // 3. Update block status to indicate body is valid
        self.statuses_store
            .write()
            .set(writer, block_hash, BlockStatus::STATUS_UTXO_VALID)
            .map_err(|_| RuleError::KnownInvalid(block_hash))?;

        Ok(())
    }
}
