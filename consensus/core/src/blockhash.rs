use jio_hashes::Hash;

/// The origin block hash in the BlockDAG.
pub const ORIGIN: Hash = Hash::from_bytes([0u8; 32]);

pub trait BlockHashExtensions {
    fn is_origin(&self) -> bool;
}

impl BlockHashExtensions for Hash {
    #[inline(always)]
    fn is_origin(&self) -> bool {
        *self == ORIGIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin() {
        assert!(ORIGIN.is_origin());
        assert!(!Hash::from_bytes([1u8; 32]).is_origin());
    }
}
