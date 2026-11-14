// script_builder.rs
//
// Builder pattern for constructing standard locking (ScriptPublicKey)
// and unlocking (SignatureScript) scripts.

use crate::opcodes::codes::*;
use smallvec::SmallVec;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptBuilder {
    script: Vec<u8>,
}

impl ScriptBuilder {
    #[inline(always)]
    pub fn new() -> Self {
        Self { script: Vec::new() }
    }

    #[inline(always)]
    pub fn add_op(mut self, op: u8) -> Self {
        self.script.push(op);
        self
    }

    /// Appends arbitrary data push:
    /// - 0 bytes -> OP_0
    /// - 1..=75 bytes -> [len, data...]
    /// - 76..=255 bytes -> [OP_PUSHDATA1, len, data...]
    pub fn add_data(mut self, data: &[u8]) -> Self {
        let len = data.len();
        if len == 0 {
            self.script.push(OP_0);
        } else if len <= 75 {
            self.script.push(len as u8);
            self.script.extend_from_slice(data);
        } else if len <= 255 {
            self.script.push(OP_PUSHDATA1);
            self.script.push(len as u8);
            self.script.extend_from_slice(data);
        } else {
            self.script.push(OP_PUSHDATA2);
            self.script.extend_from_slice(&(len as u16).to_le_bytes());
            self.script.extend_from_slice(data);
        }
        self
    }

    #[inline(always)]
    pub fn drain(self) -> Vec<u8> {
        self.script
    }

    #[inline(always)]
    pub fn to_smallvec(self) -> SmallVec<[u8; 36]> {
        SmallVec::from_vec(self.script)
    }
}
