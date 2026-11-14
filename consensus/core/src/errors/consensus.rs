use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConsensusError {
    #[error("Internal consensus storage error: {0}")]
    StorageError(String),

    #[error("Consensus service shutdown")]
    ServiceTerminated,
}
