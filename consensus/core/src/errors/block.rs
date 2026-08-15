use jio_hashes::Hash;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum BlockRuleError {
    #[error("block header version {0} is invalid")]
    BadVersion(u16),
    #[error("block timestamp {0} is in the future")]
    TimeTooNew(u64),
    #[error("block timestamp {0} is older than median time past {1}")]
    TimeTooOld(u64, u64),
    #[error("block parent count {0} exceeds max allowed {1}")]
    TooManyParents(usize, usize),
    #[error("block has no parents")]
    NoParents,
    #[error("block contains duplicate parents")]
    DuplicateParent,
    #[error("block mass {0} exceeds maximum block mass {1}")]
    ExceedsMassLimit(u64, u64),
    #[error("merkle root mismatch: expected {expected}, calculated {actual}")]
    BadMerkleRoot { expected: Hash, actual: Hash },
    #[error("UTXO commitment mismatch")]
    BadUtxoCommitment,
    #[error("block contains invalid transaction: {0}")]
    InvalidTx(String),
    #[error("block blue score {0} violates finality depth against pruning point blue score {1}")]
    FinalityViolation(u64, u64),
}
