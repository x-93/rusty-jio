use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    #[error("Invalid network configuration: {0}")]
    InvalidNetwork(String),
}
