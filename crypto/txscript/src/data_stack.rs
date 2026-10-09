//! Forth-like data stack machine primitives for script execution.

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum StackError {
    #[error("stack underflow: attempted to pop from empty stack")]
    StackUnderflow,
    #[error("invalid stack index: {0}")]
    InvalidIndex(usize),
    #[error("integer decode error")]
    InvalidInteger,
    #[error("stack item exceeds maximum allowed size")]
    ElementTooLarge,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

    pub fn push(&mut self, item: Vec<u8>) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Result<Vec<u8>, StackError> {
        self.items.pop().ok_or(StackError::StackUnderflow)
    }

    pub fn peek(&self) -> Result<&[u8], StackError> {
        self.items
            .last()
            .map(|v| v.as_slice())
            .ok_or(StackError::StackUnderflow)
    }

    pub fn peek_mut(&mut self) -> Result<&mut Vec<u8>, StackError> {
        self.items.last_mut().ok_or(StackError::StackUnderflow)
    }

    pub fn drop(&mut self) -> Result<(), StackError> {
        self.pop()?;
        Ok(())
    }

    pub fn dup(&mut self) -> Result<(), StackError> {
        let item = self.peek()?.to_vec();
        self.push(item);
        Ok(())
    }

    pub fn swap(&mut self) -> Result<(), StackError> {
        let len = self.items.len();
        if len < 2 {
            return Err(StackError::StackUnderflow);
        }
        self.items.swap(len - 1, len - 2);
        Ok(())
    }

    pub fn rot(&mut self) -> Result<(), StackError> {
        // (x1 x2 x3 -> x2 x3 x1)
        let len = self.items.len();
        if len < 3 {
            return Err(StackError::StackUnderflow);
        }
        let top = self.items.remove(len - 3);
        self.items.push(top);
        Ok(())
    }

    pub fn nip(&mut self) -> Result<(), StackError> {
        // (x1 x2 -> x2)
        let len = self.items.len();
        if len < 2 {
            return Err(StackError::StackUnderflow);
        }
        self.items.remove(len - 2);
        Ok(())
    }

    pub fn over(&mut self) -> Result<(), StackError> {
        // (x1 x2 -> x1 x2 x1)
        let len = self.items.len();
        if len < 2 {
            return Err(StackError::StackUnderflow);
        }
        let item = self.items[len - 2].clone();
        self.push(item);
        Ok(())
    }

    pub fn tuck(&mut self) -> Result<(), StackError> {
        // (x1 x2 -> x2 x1 x2)
        let len = self.items.len();
        if len < 2 {
            return Err(StackError::StackUnderflow);
        }
        let top = self.peek()?.to_vec();
        self.items.insert(len - 2, top);
        Ok(())
    }

    pub fn pick(&mut self, n: usize) -> Result<(), StackError> {
        let len = self.items.len();
        if n >= len {
            return Err(StackError::InvalidIndex(n));
        }
        let item = self.items[len - 1 - n].clone();
        self.push(item);
        Ok(())
    }

    pub fn roll(&mut self, n: usize) -> Result<(), StackError> {
        let len = self.items.len();
        if n >= len {
            return Err(StackError::InvalidIndex(n));
        }
        let item = self.items.remove(len - 1 - n);
        self.push(item);
        Ok(())
    }

    pub fn pop_bool(&mut self) -> Result<bool, StackError> {
        let bytes = self.pop()?;
        Ok(cast_to_bool(&bytes))
    }

    pub fn push_bool(&mut self, val: bool) {
        if val {
            self.push(vec![1]);
        } else {
            self.push(vec![]);
        }
    }

    pub fn pop_i64(&mut self) -> Result<i64, StackError> {
        let bytes = self.pop()?;
        decode_script_num(&bytes)
    }

    pub fn push_i64(&mut self, val: i64) {
        self.push(encode_script_num(val));
    }
}

pub fn cast_to_bool(data: &[u8]) -> bool {
    for (i, &b) in data.iter().enumerate() {
        if b != 0 {
            // Check for negative zero: [0x80] or [..., 0x80]
            if i == data.len() - 1 && b == 0x80 {
                return false;
            }
            return true;
        }
    }
    false
}

pub fn decode_script_num(bytes: &[u8]) -> Result<i64, StackError> {
    if bytes.is_empty() {
        return Ok(0);
    }
    if bytes.len() > 8 {
        return Err(StackError::InvalidInteger);
    }

    let mut result: i64 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        result |= (b as i64) << (8 * i);
    }

    // Check sign bit on most significant byte
    let msb = bytes[bytes.len() - 1];
    if (msb & 0x80) != 0 {
        // Mask out sign bit and negate
        result &= !(0x80i64 << (8 * (bytes.len() - 1)));
        Ok(-result)
    } else {
        Ok(result)
    }
}

pub fn encode_script_num(val: i64) -> Vec<u8> {
    if val == 0 {
        return Vec::new();
    }

    let is_neg = val < 0;
    let mut abs_val = val.unsigned_abs();
    let mut result = Vec::new();

    while abs_val > 0 {
        result.push((abs_val & 0xff) as u8);
        abs_val >>= 8;
    }

    // If highest bit is set, add extra sign byte
    if (result.last().unwrap() & 0x80) != 0 {
        result.push(if is_neg { 0x80 } else { 0x00 });
    } else if is_neg {
        let last = result.last_mut().unwrap();
        *last |= 0x80;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_operations() {
        let mut stack = DataStack::new();
        stack.push_i64(10);
        stack.push_i64(20);
        stack.dup().unwrap();
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop_i64().unwrap(), 20);
        assert_eq!(stack.pop_i64().unwrap(), 20);
        assert_eq!(stack.pop_i64().unwrap(), 10);
    }

    #[test]
    fn test_script_num_encoding() {
        assert_eq!(encode_script_num(0), Vec::<u8>::new());
        assert_eq!(encode_script_num(1), vec![1]);
        assert_eq!(encode_script_num(-1), vec![0x81]);
        assert_eq!(decode_script_num(&[]).unwrap(), 0);
        assert_eq!(decode_script_num(&[1]).unwrap(), 1);
        assert_eq!(decode_script_num(&[0x81]).unwrap(), -1);
        assert_eq!(decode_script_num(&encode_script_num(123456789)).unwrap(), 123456789);
        assert_eq!(decode_script_num(&encode_script_num(-123456789)).unwrap(), -123456789);
    }
}
