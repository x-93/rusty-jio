use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum PruningImportError {
    #[error("pruning proof is empty")]
    ProofIsEmpty,

    #[error("invalid pruning point: {0}")]
    InvalidPruningPoint(String),

    #[error("pruning import error: {0}")]
    Other(String),
}