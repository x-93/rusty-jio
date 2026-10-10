use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum DifficultyError {
    #[error("difficulty window is too small: {0} < {1}")]
    UnderMinDifficultyWindowSize(usize, usize),

    #[error("target is negative")]
    NegativeTarget,

    #[error("target exceeds maximum: {0} > {1}")]
    TargetTooHigh(String, String),

    #[error("difficulty adjustment error: {0}")]
    Other(String),
}