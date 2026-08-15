use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("invalid data format: {0}")]
    InvalidData(String),
}
