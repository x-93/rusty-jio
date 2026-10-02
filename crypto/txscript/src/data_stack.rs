//! Stack machine for script evaluation.

use jio_txscript_errors::ScriptError;

pub const MAX_STACK_SIZE: usize = 2048;
pub const MAX_SCRIPT_ELEMENT_SIZE: usize = 520;

#[derive(Default, Clone, Debug)]
pub struct DataStack {
    items: Vec<Vec<u8>>,
}

impl DataStack {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push(&mut self, item: Vec<u8>) -> Result<(), ScriptError> {
        if self.items.len() >= MAX_STACK_SIZE {
            return Err(ScriptError::StackOverflow);
        }
        if item.len() > MAX_SCRIPT_ELEMENT_SIZE {
            return Err(ScriptError::ElementTooLarge(item.len(), MAX_SCRIPT_ELEMENT_SIZE));
        }
        self.items.push(item);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Vec<u8>, ScriptError> {
        self.items.pop().ok_or(ScriptError::StackUnderflow)
    }

    pub fn top(&self) -> Result<&[u8], ScriptError> {
        self.items.last().map(|v| v.as_slice()).ok_or(ScriptError::StackUnderflow)
    }

    pub fn dup(&mut self) -> Result<(), ScriptError> {
        let item = self.top()?.to_vec();
        self.push(item)
    }

    pub fn drop(&mut self) -> Result<(), ScriptError> {
        self.pop()?;
        Ok(())
    }

    /// Evaluates if a byte slice represents boolean true according to consensus script rules.
    pub fn is_true(val: &[u8]) -> bool {
        for (i, &b) in val.iter().enumerate() {
            if b != 0 {
                // If the only non-zero byte is the negative zero indicator (0x80) on the last byte, it is false.
                if i == val.len() - 1 && b == 0x80 {
                    return false;
                }
                return true;
            }
        }
        false
    }
}
