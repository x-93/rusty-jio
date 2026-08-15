use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CoinbaseError {
    #[error("coinbase payload too short: {0} bytes")]
    PayloadTooShort(usize),
    #[error("coinbase payload too long: {0} bytes (max {1})")]
    PayloadTooLong(usize, usize),
    #[error("invalid coinbase subsidy: expected {expected}, got {actual}")]
    InvalidSubsidy { expected: u64, actual: u64 },
    #[error("transaction is not a coinbase transaction")]
    NotCoinbase,
    #[error("coinbase blue score mismatch: expected {1}, got {0}")]
    BlueScoreMismatch(u64, u64),
    #[error("insufficient coinbase subsidy: output total {0} < expected {1}")]
    InsufficientSubsidy(u64, u64),
}
