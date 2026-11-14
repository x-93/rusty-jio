use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DifficultyError {
    #[error("Underflow in difficulty target calculation")]
    Underflow,

    #[error("Overflow in difficulty target calculation")]
    Overflow,
}
