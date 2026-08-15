use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum DifficultyError {
    #[error("unexpected difficulty bits {expected:#x}, got {actual:#x}")]
    UnexpectedDifficulty { expected: u32, actual: u32 },
    #[error("target out of range: {0:#x}")]
    TargetOutOfRange(u32),
}
