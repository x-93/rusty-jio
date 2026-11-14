use jio_hashes::Hash;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RuleError {
    #[error("Block {0} already exists in the DAG")]
    BlockAlreadyExists(Hash),

    #[error("Block {0} is marked as invalid")]
    KnownInvalid(Hash),

    #[error("Block proof of work is invalid (does not meet target)")]
    InvalidPoW,

    #[error("Block timestamp {0} is too old (less than past median time {1})")]
    TimeTooOld(u64, u64),

    #[error("Block timestamp {0} is too far in the future")]
    TimeTooMuchInTheFuture(u64),

    #[error("Block has no parents")]
    NoParents,

    #[error("Block parent {0} not found in DAG")]
    MissingParent(Hash),

    #[error("Merkle root mismatch")]
    BadMerkleRoot,

    #[error("UTXO commitment mismatch")]
    BadUtxoCommitment,

    #[error("Block mass {0} exceeds maximum allowed mass {1}")]
    ExceedsMaxBlockMass(u64, u64),
}

pub type BlockProcessResult<T> = Result<T, RuleError>;
