use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum ConfigError {
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("config error: {0}")]
    Other(String),
}
