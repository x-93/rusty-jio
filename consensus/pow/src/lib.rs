pub mod matrix;
pub mod wasm;
pub mod xoshiro;

use jio_consensus_core::config::constants::consensus::MAX_DIFFICULTY_TARGET;
use jio_consensus_core::hashing;
use jio_consensus_core::header::Header;
use jio_consensus_core::BlueWorkType;
use jio_hashes::Hash;
use jio_math::{Uint192, Uint256};
pub use matrix::{calculate_pow, Matrix, MATRIX_SIZE};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("proof of work target exceeds maximum")]
    TargetTooHigh,
}

pub fn calc_target(bits: u32) -> Uint256 {
    Uint256::from_compact_target_bits(bits)
}

pub fn calc_work(target: Uint256) -> BlueWorkType {
    if target == Uint256::ZERO {
        return BlueWorkType::MAX;
    }
    let work = Uint256::MAX / (target + Uint256::from_u64(1));
    Uint192::try_from(work).unwrap_or(Uint192::MAX)
}

/// Matrix-based PoW hasher for the mining algorithm.
#[derive(Clone, Debug)]
pub struct PowHash {
    pub matrix: Matrix,
    pub pre_pow_hash: Hash,
}

impl PowHash {
    pub fn new(pre_pow_hash: Hash) -> Self {
        let matrix = Matrix::generate(pre_pow_hash);
        Self { matrix, pre_pow_hash }
    }

    #[inline(always)]
    pub fn calculate_pow(&self, nonce: u64) -> Hash {
        calculate_pow(&self.matrix, self.pre_pow_hash, nonce)
    }
}

/// Blake3-optimized matrix PoW hasher.
#[derive(Clone, Debug)]
pub struct PowB3Hash {
    inner: PowHash,
}

impl PowB3Hash {
    pub fn new(pre_pow_hash: Hash) -> Self {
        Self {
            inner: PowHash::new(pre_pow_hash),
        }
    }

    #[inline(always)]
    pub fn calculate_pow(&self, nonce: u64) -> Hash {
        self.inner.calculate_pow(nonce)
    }
}

pub struct State {
    pub matrix: Matrix,
    pub target: Uint256,
    pub pre_pow_hash: Hash,
}

impl State {
    pub fn new(header: &Header) -> Result<Self, Error> {
        let target = calc_target(header.bits);
        if target > MAX_DIFFICULTY_TARGET {
            return Err(Error::TargetTooHigh);
        }
        let pre_pow_hash = hashing::header::pre_pow_hash(header);
        let matrix = Matrix::generate(pre_pow_hash);
        Ok(Self {
            matrix,
            target,
            pre_pow_hash,
        })
    }

    #[inline(always)]
    pub fn calculate_pow(&self, nonce: u64) -> Hash {
        calculate_pow(&self.matrix, self.pre_pow_hash, nonce)
    }

    pub fn check_pow(&self, nonce: u64) -> (bool, BlueWorkType) {
        let pow_hash = self.calculate_pow(nonce);
        let pow_val = Uint256::from_be_bytes(pow_hash.as_bytes());
        let is_valid = pow_val <= self.target;
        let work = calc_work(self.target);
        (is_valid, work)
    }
}

pub fn check_pow(header: &Header) -> Result<(bool, BlueWorkType), Error> {
    let state = State::new(header)?;
    Ok(state.check_pow(header.nonce))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_target_and_work() {
        let target = calc_target(0x1d00ffff);
        assert_ne!(target, Uint256::ZERO);
        let work = calc_work(target);
        assert_ne!(work, BlueWorkType::ZERO);
    }

    #[test]
    fn test_pow_hash() {
        let pre_pow = Hash::from_u64_word(9999);
        let pow_hasher = PowHash::new(pre_pow);
        let h1 = pow_hasher.calculate_pow(1);
        let h2 = pow_hasher.calculate_pow(2);
        assert_ne!(h1, h2);
    }
}
