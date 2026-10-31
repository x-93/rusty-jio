use crate::matrix::Matrix64x64;
use crate::xoshiro::XoShiRo256PlusPlus;
use jio_consensus_core::Header;
use jio_hashes::{Hash, HeavyHash, ProofOfWorkHash};
use jio_math::Uint256;

/// Helper to convert a 32-byte hash into 64 4-bit nibbles.
pub fn hash_to_nibbles(hash: &Hash) -> [u8; 64] {
    let mut nibbles = [0u8; 64];
    for (i, &byte) in hash.as_bytes().iter().enumerate() {
        nibbles[i * 2] = (byte >> 4) & 0x0F;
        nibbles[i * 2 + 1] = byte & 0x0F;
    }
    nibbles
}

/// Helper to repack 64 4-bit nibbles back into 32 bytes.
pub fn nibbles_to_bytes(nibbles: &[u8; 64]) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = ((nibbles[i * 2] & 0x0F) << 4) | (nibbles[i * 2 + 1] & 0x0F);
    }
    bytes
}

/// Computes the complete kHeavyHash Proof-of-Work for a block header.
pub fn calculate_pow(header: &Header) -> Uint256 {
    // 1. Compute PrePowHash
    let pre_pow_hash = header.compute_pre_pow_hash();

    // 2. Generate 64x64 matrix from PRNG seeded by PrePowHash
    let mut prng = XoShiRo256PlusPlus::from_seed(pre_pow_hash.as_bytes());
    let matrix = Matrix64x64::generate(&mut prng);

    // 3. Digest header with timestamp and nonce
    let mut hasher = ProofOfWorkHash::new();
    hasher.update(pre_pow_hash.as_bytes());
    hasher.update(header.timestamp.to_le_bytes());
    hasher.update(header.nonce.to_le_bytes());
    let initial_hash = hasher.finalize();

    // 4. Matrix-vector multiplication over 64 nibbles
    let vector = hash_to_nibbles(&initial_hash);
    let multiplied_vector = matrix.multiply(&vector);
    let mixed_bytes = nibbles_to_bytes(&multiplied_vector);

    // 5. Final HeavyHash pass
    let mut heavy_hasher = HeavyHash::new();
    heavy_hasher.update(mixed_bytes);
    let final_hash = heavy_hasher.finalize();

    // 6. Convert to 256-bit integer
    Uint256::from_le_bytes(final_hash.as_bytes())
}

/// Verifies whether the header's Proof-of-Work satisfies its difficulty target bits.
pub fn check_pow(header: &Header) -> bool {
    let pow_val = calculate_pow(header);
    let target = Uint256::from_compact_target_bits(header.bits);
    pow_val <= target
}
