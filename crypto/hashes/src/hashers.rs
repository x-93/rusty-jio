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


/// Primary Blake3 domain-separated hasher generator macro.
#[macro_export]
macro_rules! blake3_hasher {
    ($(struct $name:ident => $domain_sep:literal),+ $(,)? ) => {$(
        #[derive(Clone)]
        pub struct $name($crate::blake3::Hasher);

        impl $name {
            #[inline(always)]
            pub fn new() -> Self {
                const KEY: [u8; $crate::blake3::KEY_LEN] = {
                    let mut key = [0u8; $crate::blake3::KEY_LEN];
                    let domain = $domain_sep;
                    let mut i = 0usize;
                    while i < domain.len() && i < key.len() {
                        key[i] = domain[i];
                        i += 1;
                    }
                    key
                };

                Self($crate::blake3::Hasher::new_keyed(&KEY))
            }

            pub fn write<A: AsRef<[u8]>>(&mut self, data: A) {
                self.0.update(data.as_ref());
            }

            #[inline(always)]
            pub fn finalize(self) -> $crate::Hash {
                let mut out = [0u8; 32];
                out.copy_from_slice(self.0.finalize().as_bytes());
                $crate::Hash::from_bytes(out)
            }
        }
    $crate::impl_hasher!{ struct $name }
    )*};
}

#[macro_export]
macro_rules! impl_hasher {
    (struct $name:ident) => {
        impl $crate::HasherBase for $name {
            #[inline(always)]
            fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
                self.write(data);
                self
            }
        }
        impl $crate::Hasher for $name {
            #[inline(always)]
            fn finalize(self) -> $crate::Hash {
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

blake3_hasher! {
    struct TransactionHash => b"TransactionHash",
    struct TransactionID => b"TransactionID",
    struct TransactionSigningHash => b"TransactionSigningHash",
    struct TransactionSigningHashECDSA => b"TransactionSigningHashECDSA",
    struct BlockHash => b"BlockHash",
    struct ProofOfWorkHash => b"ProofOfWorkHash",
    struct MerkleBranchHash => b"MerkleBranchHash",
    struct MuHashElementHash => b"MuHashElement",
    struct MuHashFinalizeHash => b"MuHashFinalize",
    struct PersonalMessageSigningHash => b"PersonalMessageSigningHash",
    struct CovenantID => b"CovenantID",

    struct SeqCommitMerkleBranch => b"SeqCommitmentMerkleBranchHash",
    struct PayloadDigest => b"PayloadDigest",
    struct TransactionRest => b"TransactionRest",
    struct TransactionV1Id => b"TransactionV1Id",
    struct SeqCommitLaneKey => b"SeqCommitLaneKey",
    struct SeqCommitLaneTip => b"SeqCommitLaneTip",
    struct SeqCommitActivityLeaf => b"SeqCommitActivityLeaf",
    struct SeqCommitMergesetContext => b"SeqCommitMergesetContext",
    struct SeqCommitMinerPayloadLeaf => b"SeqCommitMinerPayloadLeaf",
    struct SeqCommitActivityRoot => b"SeqCommitActivityRoot",

    struct SeqCommitActiveLeaf => b"SeqCommitActiveLeaf",
    struct SeqCommitActiveNode => b"SeqCommitActiveNode",
    struct SeqCommitActiveCollapsedNode => b"SeqCommitActiveCollapsedNode",
}

/// Computes a standard 32-byte Blake3 hash of input data.
pub fn blake3(data: &[u8]) -> crate::Hash {
    let digest = blake3::hash(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(digest.as_bytes());
    crate::Hash::from_bytes(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test_vector<H: Hasher>(input_data: &[&[u8]], hasher_new: impl FnOnce() -> H) {
        let mut hasher = hasher_new();
        for data in input_data {
            let hash = hasher.update(data).clone().finalize();
            assert_ne!(hash, crate::ZERO_HASH);
        }
    }

    #[test]
    fn test_blake3_all_hashers() {
        let input_data = [
            &[][..],
            &[1][..],
            &[
                5, 199, 126, 44, 71, 32, 82, 139, 122, 217, 43, 48, 52, 112, 40, 209, 180, 83, 139, 231, 72, 48, 136, 48, 168, 226,
                133, 7, 60, 4, 160, 205,
            ][..],
            &[42; 64],
            &[0; 8][..],
        ];

        run_test_vector(&input_data, TransactionHash::new);
        run_test_vector(&input_data, TransactionID::new);
        run_test_vector(&input_data, TransactionSigningHash::new);
        run_test_vector(&input_data, TransactionSigningHashECDSA::new);
        run_test_vector(&input_data, BlockHash::new);
        run_test_vector(&input_data, ProofOfWorkHash::new);
        run_test_vector(&input_data, MerkleBranchHash::new);
        run_test_vector(&input_data, SeqCommitLaneKey::new);
        run_test_vector(&input_data, PayloadDigest::new);
    }
}
