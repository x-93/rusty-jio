use crate::tx::TransactionOutpoint;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum TxRuleError {
    #[error("Transaction is missing input spending {0:?}")]
    MissingTxOut(TransactionOutpoint),

    #[error("Transaction mass {0} exceeds max allowed {1}")]
    ExceedsMassLimit(u64, u64),

    #[error("Transaction signature verification failed at input {0}")]
    InvalidSignature(usize),

    #[error("Transaction fee below minimum")]
    InsufficientFee,

    #[error("Coinbase transaction in non-coinbase position")]
    UnexpectedCoinbase,

    #[error("Immature coinbase output spent (DAA score {0} < maturity {1})")]
    ImmatureCoinbaseSpend(u64, u64),
}

pub type TxResult<T> = Result<T, TxRuleError>;
