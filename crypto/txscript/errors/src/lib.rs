//! Error definitions for the Jio transaction script engine.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub enum ScriptError {
    #[error("Script failed verification: top of stack is false or empty")]
    ScriptFailed,
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Stack overflow: exceeded max execution stack depth")]
    StackOverflow,
    #[error("Invalid opcode: 0x{0:02x}")]
    InvalidOpcode(u8),
    #[error("Disabled opcode: 0x{0:02x}")]
    DisabledOpcode(u8),
    #[error("Element too large: length {0} exceeds max allowed {1}")]
    ElementTooLarge(usize, usize),
    #[error("Script too large: length {0} exceeds max allowed {1}")]
    ScriptTooLarge(usize, usize),
    #[error("Invalid signature format")]
    InvalidSignature,
    #[error("Invalid public key format")]
    InvalidPublicKey,
    #[error("Non-clean stack: more than one element remaining on stack")]
    CleanStack,
}
