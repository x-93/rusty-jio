use crate::tx::TransactionId;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum BlockRuleError {
    #[error("block version is unknown: {0}")]
    UnknownBlockVersion(u16),

    #[error("invalid transactions in new block: {0:?}")]
    InvalidTransactionsInNewBlock(Vec<TransactionId>),

    #[error("block mass exceeds maximum: {0} > {1}")]
    BlockMassTooHigh(u64, u64),

    #[error("bad merkle root: expected {0}, actual {1}")]
    BadMerkleRoot(jio_hashes::Hash, jio_hashes::Hash),

    #[error("block has too many parents: {0} > {1}")]
    TooManyParents(usize, usize),

    #[error("block timestamp is in the future: {0} > {1}")]
    TimeTooFarInFuture(u64, u64),

    #[error("block difficulty target is invalid")]
    InvalidPoW,

    #[error("block rule error: {0}")]
    Other(String),
}

pub type RuleResult<T> = Result<T, BlockRuleError>;
