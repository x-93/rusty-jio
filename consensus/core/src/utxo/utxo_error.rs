//! UTXO diff error definitions.

use thiserror::Error;
use super::super::tx::TransactionOutpoint;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UtxoDiffError {
    #[error("Outpoint {0:?} was removed multiple times")]
    DuplicateRemove(TransactionOutpoint),
    #[error("Outpoint {0:?} was added multiple times")]
    DuplicateAdd(TransactionOutpoint),
    #[error("Incompatible diff composition")]
    IncompatibleDiff,
}
