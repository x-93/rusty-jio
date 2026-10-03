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

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConsensusError {
    #[error("Rule error: {0}")]
    RuleError(#[from] RuleError),
    #[error("Sync error: {0}")]
    SyncError(#[from] super::sync::SyncError),
    #[error("Traversal error: {0}")]
    TraversalError(#[from] super::traversal::TraversalError),
    #[error("Pruning error: {0}")]
    PruningError(#[from] super::pruning::PruningError),
    #[error("Block error: {0}")]
    BlockError(#[from] super::block::BlockError),
    #[error("Difficulty error: {0}")]
    DifficultyError(#[from] super::difficulty::DifficultyError),
    #[error("Config error: {0}")]
    ConfigError(#[from] super::config::ConfigError),
}

pub type ConsensusResult<T> = Result<T, ConsensusError>;
