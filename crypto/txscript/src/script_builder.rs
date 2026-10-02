//! Fluent builder for constructing pay-to-pubkey, multisig, and custom scripts.

use super::opcodes::macros::*;

#[derive(Default, Clone, Debug)]
pub struct ScriptBuilder {
    bytes: Vec<u8>,
}

impl ScriptBuilder {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { bytes: Vec::with_capacity(capacity) }
    }

    pub fn add_op(mut self, op: u8) -> Self {
        self.bytes.push(op);
        self
    }

    pub fn add_ops(mut self, ops: &[u8]) -> Self {
        self.bytes.extend_from_slice(ops);
        self
    }

    pub fn add_data(mut self, data: &[u8]) -> Self {
        let len = data.len();
        if len == 0 {
            self.bytes.push(OP_0);
        } else if len <= 75 {
            self.bytes.push(len as u8);
            self.bytes.extend_from_slice(data);
        } else if len <= 255 {
            self.bytes.push(OP_PUSHDATA1);
            self.bytes.push(len as u8);
            self.bytes.extend_from_slice(data);
        } else if len <= 65535 {
            self.bytes.push(OP_PUSHDATA2);
            self.bytes.extend_from_slice(&(len as u16).to_le_bytes());
            self.bytes.extend_from_slice(data);
        } else {
            self.bytes.push(OP_PUSHDATA4);
            self.bytes.extend_from_slice(&(len as u32).to_le_bytes());
            self.bytes.extend_from_slice(data);
        }
        self
    }

    pub fn add_p2pk(self, pubkey: &[u8]) -> Self {
        self.add_data(pubkey).add_op(OP_CHECKSIG)
    }

    pub fn add_p2sh(self, script_hash: &[u8]) -> Self {
        self.add_op(OP_HASH256).add_data(script_hash).add_op(OP_EQUAL)
    }

    pub fn drain(self) -> Vec<u8> {
        self.bytes
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}
