use crate::processes::transaction_validator::TxValidationError;
use consensus_core::errors::block::RuleError;
use jio_database::prelude::StoreError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VirtualStateError {
    #[error("Rule error: {0}")]
    Rule(#[from] RuleError),

    #[error("Transaction validation error: {0}")]
    Tx(#[from] TxValidationError),

    #[error("Storage error: {0}")]
    Store(#[from] StoreError),

    #[error("Calculated UTXO commitment does not match header")]
    BadUtxoCommitment,
}

pub type VirtualResult<T> = Result<T, VirtualStateError>;
