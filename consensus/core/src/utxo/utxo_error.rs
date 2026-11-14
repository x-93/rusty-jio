use crate::tx::TransactionOutpoint;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum UtxoAlgebraError {
    #[error("Outpoint {0:?} already exists in UTXO collection")]
    DuplicateOutpoint(TransactionOutpoint),

    #[error("Outpoint {0:?} not found in UTXO collection")]
    MissingOutpoint(TransactionOutpoint),
}
