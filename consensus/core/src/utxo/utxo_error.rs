use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum UtxoError {
    #[error("UTXO not found")]
    NotFound,
    #[error("duplicate UTXO entry")]
    Duplicate,
    #[error("UTXO difference conflict")]
    DiffConflict,
}
