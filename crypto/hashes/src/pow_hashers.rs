use crate::Hash;
use crate::hashers::{Hasher, HasherBase};

/// PoW Hash state based on BLAKE3 domain-separated key derivation "ProofOfWorkHash".
#[derive(Clone)]
pub struct PowHash(blake3::Hasher);

impl PowHash {
    #[inline(always)]
    pub fn new() -> Self {
        Self(blake3::Hasher::new_derive_key("ProofOfWorkHash"))
    }

    #[inline(always)]
    pub fn write<A: AsRef<[u8]>>(&mut self, data: A) {
        self.0.update(data.as_ref());
    }

    #[inline(always)]
    pub fn finalize(self) -> Hash {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.0.finalize().as_bytes());
        Hash(out)
    }
}

impl Default for PowHash {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl HasherBase for PowHash {
    #[inline(always)]
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
        self.write(data);
        self
    }
}

impl Hasher for PowHash {
    #[inline(always)]
    fn finalize(self) -> Hash {
        PowHash::finalize(self)
    }

    #[inline(always)]
    fn reset(&mut self) {
        *self = Self::new();
    }
}

/// HeavyHash state based on BLAKE3 domain-separated key derivation "HeavyHash".
#[derive(Clone)]
pub struct KHeavyHash(blake3::Hasher);

impl KHeavyHash {
    #[inline(always)]
    pub fn new() -> Self {
        Self(blake3::Hasher::new_derive_key("HeavyHash"))
    }

    #[inline(always)]
    pub fn write<A: AsRef<[u8]>>(&mut self, data: A) {
        self.0.update(data.as_ref());
    }

    #[inline(always)]
    pub fn finalize(self) -> Hash {
        let mut out = [0u8; 32];
        out.copy_from_slice(self.0.finalize().as_bytes());
        Hash(out)
    }
}

impl Default for KHeavyHash {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl HasherBase for KHeavyHash {
    #[inline(always)]
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
        self.write(data);
        self
    }
}

impl Hasher for KHeavyHash {
    #[inline(always)]
    fn finalize(self) -> Hash {
        KHeavyHash::finalize(self)
    }

    #[inline(always)]
    fn reset(&mut self) {
        *self = Self::new();
    }
}
