pub mod calc;
pub mod matrix;
pub mod xoshiro;

pub use calc::{calculate_pow, check_pow, hash_to_nibbles, nibbles_to_bytes};
pub use matrix::Matrix64x64;
pub use xoshiro::XoShiRo256PlusPlus;

#[cfg(test)]
mod tests {
    use super::*;
    use jio_consensus_core::Header;
    use jio_hashes::Hash;
    use jio_math::Uint192;

    #[test]
    fn test_xoshiro256plusplus_sequence() {
        let mut prng = XoShiRo256PlusPlus::from_seed([1u8; 32]);
        let val1 = prng.next_u64();
        let val2 = prng.next_u64();
        assert_ne!(val1, 0);
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_matrix_vector_multiplication() {
        let mut prng = XoShiRo256PlusPlus::from_seed([42u8; 32]);
        let matrix = Matrix64x64::generate(&mut prng);

        let input_vec = [7u8; 64];
        let out_vec = matrix.multiply(&input_vec);

        // Output nibbles must stay strictly within 0..=15
        for &nibble in &out_vec {
            assert!(nibble <= 15);
        }
    }

    #[test]
    fn test_nibble_roundtrip_conversion() {
        let hash = Hash::from_u64_word(0x123456789ABCDEF0);
        let nibbles = hash_to_nibbles(&hash);
        let recovered = nibbles_to_bytes(&nibbles);
        assert_eq!(hash.as_bytes(), recovered);
    }

    #[test]
    fn test_pow_verification_with_easy_target() {
        // High target (easiest difficulty)
        let easy_bits = 0x207fffff;
        let mut header = Header::new(
            1,
            vec![],
            Hash::ZERO,
            Hash::ZERO,
            Hash::ZERO,
            1_700_000_000_000,
            easy_bits,
            0,
            0,
            Uint192::ZERO,
            0,
            Hash::ZERO,
        );

        // Mine a nonce that passes
        let mut found = false;
        for nonce in 0..10_000 {
            header.nonce = nonce;
            if check_pow(&header) {
                found = true;
                break;
            }
        }
        assert!(found, "Should find valid PoW with easy target");
    }
}
