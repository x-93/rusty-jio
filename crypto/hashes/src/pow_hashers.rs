//! Proof-of-work hashers and state matrices.

use super::Hash;
use sha2::Sha256;
use sha2::Digest;

/// Proof-of-work pre-hash state.
#[derive(Clone, Default)]
pub struct PowHasher {
    hasher: Sha256,
}

impl PowHasher {
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    #[inline(always)]
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.hasher.update(data);
        self
    }

    #[inline(always)]
    pub fn finalize(self) -> Hash {
        let result = self.hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Hash(bytes)
    }

    pub fn hash(data: &[u8]) -> Hash {
        let mut h = Self::new();
        h.update(data);
        h.finalize()
    }
}

impl crate::hashers::HasherBase for PowHasher {
    #[inline(always)]
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
        self.update(data.as_ref())
    }
}

impl crate::hashers::Hasher for PowHasher {
    #[inline(always)]
    fn finalize(self) -> Hash {
        self.finalize()
    }
    #[inline(always)]
    fn reset(&mut self) {
        *self = Self::new();
    }
}

pub type PowHash = PowHasher;
