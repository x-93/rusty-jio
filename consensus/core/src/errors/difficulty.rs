use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DifficultyError {
    #[error("Difficulty target exceeds maximum")]
    TargetTooHigh,
    #[error("Unexpected difficulty target value")]
    UnexpectedTarget,
}
