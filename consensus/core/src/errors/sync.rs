use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum SyncError {
    #[error("sync negotiation failed: {0}")]
    NegotiationFailed(String),

    #[error("sync error: {0}")]
    Other(String),
}
