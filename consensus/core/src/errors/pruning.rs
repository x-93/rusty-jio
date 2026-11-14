use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PruningError {
    #[error("Pruning point not found")]
    PruningPointNotFound,

    #[error("Past pruning point")]
    PastPruningPoint,
}
