use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    #[error("key not found in store")]
    KeyNotFound(String),
    #[error("key already exists in store")]
    KeyAlreadyExists(String),
    #[error("data corruption: {0}")]
    CorruptedData(String),
    #[error("internal database error: {0}")]
    Internal(String),
    #[error("operation not supported: {0}")]
    NotSupported(String),
}

pub type StoreResult<T> = Result<T, StoreError>;
pub type DbResult<T> = Result<T, StoreError>;
