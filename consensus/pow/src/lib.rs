#![allow(clippy::arithmetic_side_effects)]

// public for benchmarks
#[doc(hidden)]
pub mod matrix;
#[doc(hidden)]
pub mod xoshiro;

use std::cmp::max;

use crate::matrix::Matrix;
use jio_consensus_core::{hashing, header::Header, BlockLevel};
use jio_hashes::PowHash;
use jio_math::Uint256;

/// State is an intermediate data structure with pre-computed values to speed up mining.
pub struct State {
    pub(crate) matrix: Matrix,
    pub(crate) target: Uint256,
    // PRE_POW_HASH || TIME || 32 zero byte padding; without NONCE
    pub(crate) hasher: PowHash,
}

impl State {
    #[inline]
    pub fn new(header: &Header) -> Self {
        let target = Uint256::from_compact_target_bits(header.bits);
        // Zero out the time and nonce.
        let pre_pow_hash = hashing::header::hash_override_nonce_time(header, 0, 0);
        // PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
        let hasher = PowHash::new(pre_pow_hash, header.timestamp);
        let matrix = Matrix::generate(pre_pow_hash);

        Self { matrix, target, hasher }
    }

    #[inline]
    #[must_use]
    /// PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
    pub fn calculate_pow(&self, nonce: u64) -> Uint256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        let hash = self.matrix.heavy_hash(hash);
        Uint256::from_le_bytes(hash.as_bytes())
    }

    #[inline]
    #[must_use]
    pub fn check_pow(&self, nonce: u64) -> (bool, Uint256) {
        let pow = self.calculate_pow(nonce);
        // The pow hash must be less or equal than the claimed target.
        (pow <= self.target, pow)
    }
}

pub fn calc_block_level(header: &Header, max_block_level: BlockLevel) -> BlockLevel {
    let (block_level, _) = calc_block_level_check_pow(header, max_block_level);
    block_level
}

pub fn calc_block_level_check_pow(header: &Header, max_block_level: BlockLevel) -> (BlockLevel, bool) {
    if header.parents_by_level.is_empty() {
        return (max_block_level, true); // Genesis has the max block level
    }

    let state = State::new(header);
    let (passed, pow) = state.check_pow(header.nonce);
    let block_level = calc_level_from_pow(pow, max_block_level);
    (block_level, passed)
}

pub fn calc_level_from_pow(pow: Uint256, max_block_level: BlockLevel) -> BlockLevel {
    let signed_block_level = max_block_level as i64 - pow.bits() as i64;
    max(signed_block_level, 0) as BlockLevel
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_hashes::ZERO_HASH;

    #[test]
    fn test_calc_level_from_pow() {
        assert_eq!(calc_level_from_pow(Uint256::from_u64(0), 128), 128);
        assert_eq!(calc_level_from_pow(Uint256::from_u64(1), 128), 127);
        assert_eq!(calc_level_from_pow(Uint256::MAX, 128), 0);
    }

    #[test]
    fn test_pow_state_genesis() {
        let header = Header::new_finalized(
            1,
            vec![],
            ZERO_HASH,
            ZERO_HASH,
            ZERO_HASH,
            1234567890,
            0x1e7fffff,
            0,
            0,
            0.into(),
            0,
            ZERO_HASH,
        );
        let (level, passed) = calc_block_level_check_pow(&header, 128);
        assert_eq!(level, 128);
        assert!(passed);
    }
}
