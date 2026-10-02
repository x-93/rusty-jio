use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlockError {
    #[error("Block not found: {0}")]
    NotFound(jio_hashes::Hash),
    #[error("Header not found: {0}")]
    HeaderNotFound(jio_hashes::Hash),
    #[error("Invalid transaction count in block body")]
    InvalidTxCount,
    #[error("Merkle root mismatch: expected {0}, got {1}")]
    MerkleRootMismatch(jio_hashes::Hash, jio_hashes::Hash),
}
