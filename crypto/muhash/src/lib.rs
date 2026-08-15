pub mod u3072;

pub use u3072::U3072;

use jio_hashes::{Hash, MuHashElementHash, MuHashFinalizeHash};

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct MuHash {
    num: U3072,
}

impl MuHash {
    pub fn new() -> Self {
        Self { num: U3072::ONE }
    }

    pub fn add_element(&mut self, data: &[u8]) {
        let mut hasher = MuHashElementHash::new();
        hasher.write(data);
        let hash = hasher.finalize();
        let mut words = [0u64; u3072::U3072_WORDS];
        let h_bytes = hash.as_bytes();
        for i in 0..4 {
            let mut b = [0u8; 8];
            b.copy_from_slice(&h_bytes[i * 8..(i + 1) * 8]);
            words[i] = u64::from_le_bytes(b);
        }
        let element = U3072(words);
        self.num.multiply(&element);
    }

    pub fn remove_element(&mut self, data: &[u8]) {
        let mut hasher = MuHashElementHash::new();
        hasher.write(data);
        let hash = hasher.finalize();
        let mut words = [0u64; u3072::U3072_WORDS];
        let h_bytes = hash.as_bytes();
        for i in 0..4 {
            let mut b = [0u8; 8];
            b.copy_from_slice(&h_bytes[i * 8..(i + 1) * 8]);
            words[i] = u64::from_le_bytes(b);
        }
        let element = U3072(words);
        self.num.divide(&element);
    }

    pub fn finalize(&self) -> Hash {
        let bytes = self.num.to_bytes();
        let mut hasher = MuHashFinalizeHash::new();
        hasher.write(&bytes);
        hasher.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muhash_consistency() {
        let mut muhash1 = MuHash::new();
        muhash1.add_element(b"outpoint1");
        muhash1.add_element(b"outpoint2");

        let mut muhash2 = MuHash::new();
        muhash2.add_element(b"outpoint2");
        muhash2.add_element(b"outpoint1");

        // MuHash is commutative
        assert_eq!(muhash1.finalize(), muhash2.finalize());
    }
}
