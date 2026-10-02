use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TraversalError {
    #[error("Past window traversal exceeded depth limit")]
    WindowLimitExceeded,
}
