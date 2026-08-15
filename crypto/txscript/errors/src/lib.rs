use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum TxScriptError {
    #[error("invalid opcode {0:#x}")]
    InvalidOpcode(u8),
    #[error("script stack underflow")]
    StackUnderflow,
    #[error("script stack overflow")]
    StackOverflow,
    #[error("script failed evaluation")]
    ScriptFailed,
    #[error("malformed script")]
    MalformedScript,
    #[error("invalid data length for push: expected <= {expected}, got {actual}")]
    InvalidPushLength { expected: usize, actual: usize },
    #[error("invalid signature")]
    InvalidSignature,
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("invalid script class")]
    InvalidScriptClass,
}

pub type Result<T> = std::result::Result<T, TxScriptError>;
