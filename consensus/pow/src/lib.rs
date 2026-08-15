pub mod matrix;
pub mod wasm;
pub mod xoshiro;

pub use matrix::*;
pub use wasm::*;
pub use xoshiro::*;

use jio_consensus_core::header::Header;
use jio_hashes::Hash;
use jio_math::Uint256;

/// Converts a 32-bit compact difficulty representation (Bits) to a Uint256 target.
pub fn compact_to_target(bits: u32) -> Uint256 {
    let exponent = (bits >> 24) as u8;
    let mantissa = bits & 0x007f_ffff;
    if exponent <= 3 {
        Uint256::from_u64(mantissa as u64 >> (8 * (3 - exponent)))
    } else {
        Uint256::from_u64(mantissa as u64) << (8 * (exponent - 3) as usize)
    }
}

/// Converts a Uint256 target back into a 32-bit compact representation.
pub fn target_to_compact(target: Uint256) -> u32 {
    let mut exponent = 32u32;
    let bytes = target.to_be_bytes();
    let mut i = 0;
    while i < 32 && bytes[i] == 0 {
        i += 1;
        exponent -= 1;
    }

    if exponent == 0 {
        return 0;
    }

    let mut mantissa = if i <= 29 {
        u32::from_be_bytes([0, bytes[i], bytes[i + 1], bytes[i + 2]])
    } else if i == 30 {
        u32::from_be_bytes([0, 0, bytes[30], bytes[31]])
    } else {
        u32::from_be_bytes([0, 0, 0, bytes[31]])
    };

    if mantissa > 0x007f_ffff {
        mantissa >>= 8;
        exponent += 1;
    }

    (exponent << 24) | (mantissa & 0x007f_ffff)
}

/// Calculates HeavyHash for a given pre_pow_hash and nonce
pub fn calc_heavy_hash(pre_pow_hash: Hash, nonce: u64) -> Hash {
    let mut hasher = jio_hashes::ProofOfWorkHash::new();
    hasher.write(pre_pow_hash);
    hasher.write(&nonce.to_le_bytes());
    let intermediate = hasher.finalize();

    let matrix = Matrix::generate(&pre_pow_hash);
    matrix.heavy_hash(&intermediate)
}

/// Validates Proof of Work for a given block header
pub fn check_pow(header: &Header) -> bool {
    let pre_pow_hash = jio_consensus_core::hashing::header::pre_pow_hash(header);
    let pow_hash = calc_heavy_hash(pre_pow_hash, header.nonce);
    let target = compact_to_target(header.bits);
    let hash_val = Uint256::from_le_bytes(pow_hash.as_bytes());
    hash_val <= target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_to_target_roundtrip() {
        let bits = 0x1e03_ffff;
        let target = compact_to_target(bits);
        assert!(!target.is_zero());
        let back_bits = target_to_compact(target);
        assert_eq!(bits, back_bits);
    }

    #[test]
    fn test_heavy_hash_deterministic() {
        let hash = Hash::from_bytes([42u8; 32]);
        let h1 = calc_heavy_hash(hash, 100);
        let h2 = calc_heavy_hash(hash, 100);
        let h3 = calc_heavy_hash(hash, 101);
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
