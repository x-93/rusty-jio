use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CoinbaseError {
    #[error("Coinbase payload exceeds maximum size")]
    PayloadTooLarge,

    #[error("Invalid script public key in coinbase")]
    InvalidScriptPublicKey,
}
