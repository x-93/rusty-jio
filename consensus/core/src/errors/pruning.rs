use jio_hashes::Hash;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum PruningError {
    #[error("pruning point {0} is invalid")]
    InvalidPruningPoint(Hash),
    #[error("pruning point has not moved")]
    PruningPointStalled,
    #[error("invalid proof: {0}")]
    InvalidProof(String),
    #[error("past pruning point {0} is invalid")]
    InvalidPastPruningPoint(Hash),
}
