use crate::hashers::ProofOfWorkHash;
use crate::Hash;

#[derive(Clone, Default)]
pub struct PowB3Hash {
    #[allow(dead_code)]
    hasher: ProofOfWorkHash,
}

impl PowB3Hash {
    pub fn new() -> Self {
        Self {
            hasher: ProofOfWorkHash::new(),
        }
    }

    pub fn calculate(pre_pow_hash: Hash, timestamp: u64, nonce: u64) -> Hash {
        let mut hasher = ProofOfWorkHash::new();
        hasher.write(pre_pow_hash);
        hasher.write(&timestamp.to_le_bytes());
        hasher.write(&nonce.to_le_bytes());
        hasher.finalize()
    }
}

pub fn pow_hash(pre_pow_hash: Hash, timestamp: u64, nonce: u64) -> Hash {
    PowB3Hash::calculate(pre_pow_hash, timestamp, nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow_hash() {
        let pre_pow = Hash::from_bytes([1u8; 32]);
        let h1 = pow_hash(pre_pow, 12345678, 9999);
        let h2 = pow_hash(pre_pow, 12345678, 10000);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_pow_b3_hash() {
        let pre_pow = Hash::from_bytes([1u8; 32]);
        let h1 = PowB3Hash::calculate(pre_pow, 12345678, 9999);
        let h2 = PowB3Hash::calculate(pre_pow, 12345678, 10000);
        assert_ne!(h1, h2);
    }
}
