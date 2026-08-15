use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum ConfigError {
    #[error("invalid network: {0}")]
    InvalidNetwork(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
}
