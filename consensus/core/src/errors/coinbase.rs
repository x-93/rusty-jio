use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum CoinbaseError {
    #[error("coinbase output value {0} exceeds allowed subsidy + fees {1}")]
    CoinbaseTooLarge(u64, u64),

    #[error("script public key exceeds max version")]
    InvalidScriptPublicKeyVersion,

    #[error("coinbase error: {0}")]
    Other(String),
}