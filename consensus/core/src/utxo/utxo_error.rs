use crate::tx::TransactionOutpoint;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum UtxoAlgebraError {
    #[error("outpoint already exists in UTXO collection: {0:?}")]
    DuplicateAdd(TransactionOutpoint),

    #[error("outpoint does not exist in UTXO collection: {0:?}")]
    MissingRemove(TransactionOutpoint),

    #[error("general error in UTXO algebra: {0}")]
    General(String),
}
