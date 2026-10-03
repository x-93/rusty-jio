use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoinbaseError {
    #[error("Coinbase payload exceeds maximum size")]
    PayloadTooLarge,
    #[error("Coinbase subsidy amount exceeds maximum allowed")]
    SubsidyTooHigh,
}

pub type CoinbaseResult<T> = Result<T, CoinbaseError>;
