use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PruningError {
    #[error("Invalid pruning point proof")]
    InvalidProof,
    #[error("Pruning point violation: block is below pruning depth")]
    BelowPruningDepth,
}
