use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TraversalError {
    #[error("Reached maximum DAG traversal depth")]
    MaxDepthExceeded,
}
