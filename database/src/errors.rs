//! Database and storage error definitions.

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    #[error("Key not found in database: {0}")]
    KeyNotFound(String),

    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),

    #[error("Database serialization/deserialization error: {0}")]
    SerializationError(String),

    #[error("Underlying database I/O error: {0}")]
    DbError(String),

    #[error("Database lock contention error")]
    LockError,

    #[error("Storage capacity or memory limit exceeded")]
    CapacityExceeded,
}

pub type StoreResult<T> = Result<T, StoreError>;
