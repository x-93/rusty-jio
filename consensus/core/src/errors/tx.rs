use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TxError {
    #[error("Transaction has no inputs")]
    NoInputs,
    #[error("Transaction has no outputs")]
    NoOutputs,
    #[error("Output value exceeds maximum allowed")]
    AmountTooHigh,
    #[error("Duplicate input detected: {0:?}")]
    DuplicateInput(super::super::tx::TransactionOutpoint),
    #[error("Missing UTXO entry for input: {0:?}")]
    MissingUtxo(super::super::tx::TransactionOutpoint),
    #[error("Total input value {0} is less than total output value {1}")]
    InputsLessThanOutputs(u64, u64),
    #[error("Script execution failed: {0}")]
    ScriptFailed(String),
}
