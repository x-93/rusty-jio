pub mod inquirer;
pub mod interval;
pub mod tests;
pub mod tree;

pub use inquirer::*;
pub use interval::*;
pub use tree::*;

use jio_hashes::Hash;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ReachabilityError {
    #[error("missing interval for block {0}")]
    MissingInterval(Hash),
    #[error("block {0} is not a chain ancestor of {1}")]
    NotChainAncestor(Hash, Hash),
    #[error("store error: {0}")]
    StoreError(String),
}

pub type Result<T> = std::result::Result<T, ReachabilityError>;
