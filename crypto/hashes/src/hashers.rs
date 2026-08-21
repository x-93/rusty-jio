pub trait HasherBase {
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self;
}

pub trait Hasher: HasherBase + Clone + Default {
    fn finalize(self) -> crate::Hash;
    fn reset(&mut self);
    #[inline(always)]
    fn hash<A: AsRef<[u8]>>(data: A) -> crate::Hash {
        let mut hasher = Self::default();
        hasher.update(data);
        hasher.finalize()
    }
}

pub use crate::pow_hashers::{KHeavyHash, PowHash};

macro_rules! impl_hasher {
    (struct $name:ident) => {
        impl HasherBase for $name {
            #[inline(always)]
            fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
                self.write(data);
                self
            }
        }
        impl Hasher for $name {
            #[inline(always)]
            fn finalize(self) -> crate::Hash {
                $name::finalize(self)
            }
            #[inline(always)]
            fn reset(&mut self) {
                *self = Self::new();
            }
        }
        impl Default for $name {
            #[inline(always)]
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

macro_rules! blake3_hasher {
    ($(struct $name:ident => $domain_sep:literal),+ $(,)? ) => {$(
        #[derive(Clone)]
        pub struct $name(blake3::Hasher);

        impl $name {
            #[inline(always)]
            pub fn new() -> Self {
                Self(blake3::Hasher::new_derive_key($domain_sep))
            }

            #[inline(always)]
            pub fn write<A: AsRef<[u8]>>(&mut self, data: A) {
                self.0.update(data.as_ref());
            }

            #[inline(always)]
            pub fn finalize(self) -> crate::Hash {
                let mut out = [0u8; 32];
                out.copy_from_slice(self.0.finalize().as_bytes());
                crate::Hash(out)
            }
        }
        impl_hasher!{ struct $name }
    )*};
}

blake3_hasher! {
    struct TransactionHash => "TransactionHash",
    struct TransactionID => "TransactionID",
    struct TransactionSigningHash => "TransactionSigningHash",
    struct TransactionSigningHashECDSA => "TransactionSigningHashECDSA",
    struct BlockHash => "BlockHash",
    struct ProofOfWorkHash => "ProofOfWorkHash",
    struct MerkleBranchHash => "MerkleBranchHash",
    struct MuHashElementHash => "MuHashElement",
    struct MuHashFinalizeHash => "MuHashFinalize",
    struct PersonalMessageSigningHash => "PersonalMessageSigningHash",
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_separation_isolation() {
        let payload = b"sample blockchain payload";

        let h_tx = TransactionHash::hash(payload);
        let h_tx_id = TransactionID::hash(payload);
        let h_block = BlockHash::hash(payload);
        let h_pow = ProofOfWorkHash::hash(payload);
        let h_merkle = MerkleBranchHash::hash(payload);
        let h_msg = PersonalMessageSigningHash::hash(payload);

        // Every BLAKE3 derive_key domain generates completely unique outputs
        assert_ne!(h_tx, h_tx_id);
        assert_ne!(h_tx_id, h_block);
        assert_ne!(h_block, h_pow);
        assert_ne!(h_pow, h_merkle);
        assert_ne!(h_merkle, h_msg);
    }

    #[test]
    fn test_incremental_streaming() {
        let mut hasher = TransactionHash::new();
        hasher.update(b"chunk1");
        hasher.update(b"chunk2");
        let hash1 = hasher.finalize();

        let hash2 = TransactionHash::hash(b"chunk1chunk2");
        assert_eq!(hash1, hash2);
    }
}
