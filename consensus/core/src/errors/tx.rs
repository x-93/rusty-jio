use crate::tx::TransactionId;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum TxRuleError {
    #[error("transaction version is unknown: {0}")]
    UnknownVersion(u16),

    #[error("transaction has no inputs")]
    NoInputs,

    #[error("transaction has no outputs")]
    NoOutputs,

    #[error("transaction output amount overflow")]
    OutputValueOverflow,

    #[error("transaction inputs not found: {0:?}")]
    MissingInputs(TransactionId),

    #[error("transaction fee is negative: in {0} < out {1}")]
    NegativeFee(u64, u64),

    #[error("transaction mass exceeds limit: {0} > {1}")]
    MassExceedsMaximum(u64, u64),

    #[error("coinbase transaction in non-coinbase position")]
    UnexpectedCoinbase,

    #[error("tx rule error: {0}")]
    Other(String),
}

pub type TxResult<T> = Result<T, TxRuleError>;