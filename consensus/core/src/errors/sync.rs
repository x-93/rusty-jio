use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SyncError {
    #[error("Synchronization timeout")]
    Timeout,
    #[error("Sync negotiation failed: common ancestor not found")]
    NegotiationFailed,
}
