use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum MiningError {
    #[error("failed to create block template: {0}")]
    BlockCreation(String),

    #[error("transaction rejected: {0}")]
    TxRejected(String),

    #[error("transaction already in mempool: {0}")]
    TxAlreadyInMempool(String),

    #[error("mempool is full (max limit reached)")]
    MempoolFull,

    #[error("orphan transaction: missing inputs")]
    OrphanTransaction,

    #[error("fee too low: got {0}, minimum required {1}")]
    FeeTooLow(u64, u64),

    #[error("transaction mass {0} exceeded maximum limit {1}")]
    MassExceeded(u64, u64),

    #[error("consensus error during mining: {0}")]
    Consensus(String),
}

pub type MiningResult<T> = Result<T, MiningError>;
