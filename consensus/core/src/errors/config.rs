use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("Invalid configuration parameter: {0}")]
    Invalid(String),
}
