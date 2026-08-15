use crate::error::TxScriptError;
use crate::result::Result;

#[derive(Default, Debug, Clone)]
pub struct DataStack {
    stack: Vec<Vec<u8>>,
}

impl DataStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, data: Vec<u8>) {
        self.stack.push(data);
    }

    pub fn pop(&mut self) -> Result<Vec<u8>> {
        self.stack.pop().ok_or(TxScriptError::StackUnderflow)
    }

    pub fn peek(&self) -> Result<&[u8]> {
        self.stack
            .last()
            .map(|v| v.as_slice())
            .ok_or(TxScriptError::StackUnderflow)
    }

    pub fn dup(&mut self) -> Result<()> {
        let top = self.peek()?.to_vec();
        self.push(top);
        Ok(())
    }

    pub fn drop(&mut self) -> Result<()> {
        self.pop()?;
        Ok(())
    }

    pub fn swap(&mut self) -> Result<()> {
        let len = self.stack.len();
        if len < 2 {
            return Err(TxScriptError::StackUnderflow);
        }
        self.stack.swap(len - 1, len - 2);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_operations() {
        let mut stack = DataStack::new();
        assert!(stack.is_empty());

        stack.push(vec![1, 2, 3]);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.peek().unwrap(), &[1, 2, 3]);

        stack.dup().unwrap();
        assert_eq!(stack.len(), 2);

        let popped = stack.pop().unwrap();
        assert_eq!(popped, vec![1, 2, 3]);

        stack.push(vec![4, 5]);
        stack.swap().unwrap();
        assert_eq!(stack.pop().unwrap(), vec![1, 2, 3]);
        assert_eq!(stack.pop().unwrap(), vec![4, 5]);

        assert!(stack.pop().is_err());
    }
}
