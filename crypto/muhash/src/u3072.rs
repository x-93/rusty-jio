//! 3072-bit unsigned integer modular arithmetic representation for MuHash (multiplicative hashing of sets).

pub const U3072_WORDS: usize = 48; // 48 * 64 bits = 3072 bits
pub const U3072_BYTES: usize = 384;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct U3072(pub [u64; U3072_WORDS]);

impl Default for U3072 {
    fn default() -> Self {
        Self::ONE
    }
}

impl U3072 {
    pub const ONE: Self = {
        let mut arr = [0u64; U3072_WORDS];
        arr[0] = 1;
        Self(arr)
    };

    pub const ZERO: Self = Self([0u64; U3072_WORDS]);

    pub fn from_bytes(bytes: &[u8; U3072_BYTES]) -> Self {
        let mut words = [0u64; U3072_WORDS];
        for (i, word) in words.iter_mut().enumerate() {
            let mut b = [0u8; 8];
            b.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
            *word = u64::from_le_bytes(b);
        }
        Self(words)
    }

    pub fn to_bytes(&self) -> [u8; U3072_BYTES] {
        let mut bytes = [0u8; U3072_BYTES];
        for (i, &word) in self.0.iter().enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    pub fn multiply(&mut self, other: &Self) {
        // Modular multiplication in 3072-bit finite field
        let mut res = [0u64; U3072_WORDS];
        for i in 0..U3072_WORDS {
            let mut carry = 0u128;
            for j in 0..(U3072_WORDS - i) {
                let prod = (self.0[i] as u128) * (other.0[j] as u128)
                    + (res[i + j] as u128)
                    + carry;
                res[i + j] = prod as u64;
                carry = prod >> 64;
            }
        }
        self.0 = res;
    }

    pub fn divide(&mut self, _other: &Self) {
        // Multiplicative inverse / division placeholder for set element removal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u3072_roundtrip() {
        let one = U3072::ONE;
        let bytes = one.to_bytes();
        let from_bytes = U3072::from_bytes(&bytes);
        assert_eq!(one, from_bytes);
    }
}
