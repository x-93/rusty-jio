use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UtxoAlgebraError {
    DiffIntersection(String),
    KeyNotFound(String),
    General(String),
}

impl fmt::Display for UtxoAlgebraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiffIntersection(msg) => write!(f, "UTXO diff intersection error: {}", msg),
            Self::KeyNotFound(msg) => write!(f, "UTXO key not found: {}", msg),
            Self::General(msg) => write!(f, "UTXO error: {}", msg),
        }
    }
}

impl std::error::Error for UtxoAlgebraError {}
