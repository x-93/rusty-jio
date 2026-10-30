use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("RocksDB error: {0}")]
    RocksDb(#[from] rocksdb::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Database is closed or unavailable")]
    Closed,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid key: {0}")]
    InvalidKey(String),
}

