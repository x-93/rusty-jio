use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum TraversalError {
    #[error("past traversal reached limit: {0}")]
    LimitExceeded(usize),

    #[error("block not found during traversal: {0}")]
    BlockNotFound(String),

    #[error("traversal error: {0}")]
    Other(String),
}
