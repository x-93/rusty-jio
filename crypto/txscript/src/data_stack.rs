// data_stack.rs
//
// Stack data structure for TxScript virtual machine.

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum StackError {
    #[error("Stack underflow")]
    Underflow,
    #[error("Stack overflow: maximum stack depth {0} exceeded")]
    Overflow(usize),
    #[error("Element size {0} exceeds maximum allowed element size {1}")]
    ElementTooLarge(usize, usize),
    #[error("Invalid stack index: {0}")]
    InvalidIndex(usize),
}

pub const MAX_STACK_SIZE: usize = 1000;
pub const MAX_SCRIPT_ELEMENT_SIZE: usize = 520;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataStack {
    stack: Vec<Vec<u8>>,
}

impl DataStack {
    #[inline(always)]
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn push(&mut self, item: Vec<u8>) -> Result<(), StackError> {
        if self.stack.len() >= MAX_STACK_SIZE {
            return Err(StackError::Overflow(MAX_STACK_SIZE));
        }
        if item.len() > MAX_SCRIPT_ELEMENT_SIZE {
            return Err(StackError::ElementTooLarge(item.len(), MAX_SCRIPT_ELEMENT_SIZE));
        }
        self.stack.push(item);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Vec<u8>, StackError> {
        self.stack.pop().ok_or(StackError::Underflow)
    }

    pub fn peek(&self) -> Result<&[u8], StackError> {
        self.stack.last().map(|v| v.as_slice()).ok_or(StackError::Underflow)
    }

    pub fn dup(&mut self) -> Result<(), StackError> {
        let top = self.peek()?.to_vec();
        self.push(top)
    }

    pub fn drop(&mut self) -> Result<(), StackError> {
        self.pop().map(|_| ())
    }

    pub fn swap(&mut self) -> Result<(), StackError> {
        let len = self.stack.len();
        if len < 2 {
            return Err(StackError::Underflow);
        }
        self.stack.swap(len - 1, len - 2);
        Ok(())
    }
}

