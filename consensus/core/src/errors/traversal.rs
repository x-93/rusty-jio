use jio_hashes::Hash;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum TraversalError {
    #[error("block {0} not found in DAG")]
    BlockNotFound(Hash),
    #[error("reachability error: {0}")]
    Reachability(String),
}
