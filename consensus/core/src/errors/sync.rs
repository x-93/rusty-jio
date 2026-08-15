use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum SyncError {
    #[error("peer handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("sync timeout")]
    Timeout,
    #[error("invalid block locator")]
    InvalidLocator,
}
