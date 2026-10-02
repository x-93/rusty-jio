//! Consensus rule errors.

use thiserror::Error;
use super::block::BlockError;
use super::tx::TxError;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuleError {
    #[error("Block error: {0}")]
    BlockError(#[from] BlockError),
    #[error("Transaction rule error: {0}")]
    TxError(#[from] TxError),
    #[error("Missing parent: {0}")]
    MissingParent(jio_hashes::Hash),
    #[error("Block is already known: {0}")]
    KnownBlock(jio_hashes::Hash),
    #[error("Bad proof of work: {0}")]
    BadPow(jio_hashes::Hash),
    #[error("Block timestamp too old: {0} <= past median time {1}")]
    TimeTooOld(u64, u64),
    #[error("Block timestamp too far in future: {0} > max allowed {1}")]
    TimeTooNew(u64, u64),
}
