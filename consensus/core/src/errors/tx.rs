use crate::tx::TransactionId;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum TxRuleError {
    #[error("transaction {0} has no inputs")]
    NoInputs(TransactionId),
    #[error("transaction {0} has no outputs")]
    NoOutputs(TransactionId),
    #[error("transaction {0} contains duplicate inputs")]
    DuplicateInputs(TransactionId),
    #[error("transaction {0} exceeds maximum mass {1}")]
    ExceedsMassLimit(TransactionId, u64),
    #[error("transaction output value exceeds maximum sompi: {0}")]
    ValueExceedsMaxSompi(u64),
    #[error("transaction fee {0} is negative")]
    NegativeFee(i64),
    #[error("transaction is not finalized at DAA score {0}")]
    NotFinalized(u64),
    #[error("script public key size {0} exceeds limit {1}")]
    ScriptPublicKeyTooLarge(usize, usize),
    #[error("signature script verification failed: {0}")]
    ScriptFailed(String),
}
