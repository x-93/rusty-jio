//! WASM bindings and helpers for PoW calculation.

use crate::{calc_target, calculate_pow, Matrix};
use jio_hashes::Hash;
use jio_math::Uint256;

pub fn wasm_calc_target(bits: u32) -> [u64; 4] {
    let target = calc_target(bits);
    target.0
}

pub fn wasm_is_valid_target(target: &[u64; 4]) -> bool {
    let t = Uint256(*target);
    t <= jio_consensus_core::config::constants::consensus::MAX_DIFFICULTY_TARGET
}

pub fn wasm_check_pow(pre_pow_hash: &[u8; 32], bits: u32, nonce: u64) -> bool {
    let target = calc_target(bits);
    if target > jio_consensus_core::config::constants::consensus::MAX_DIFFICULTY_TARGET {
        return false;
    }
    let pph = Hash::from_bytes(*pre_pow_hash);
    let matrix = Matrix::generate(pph);
    let pow_hash = calculate_pow(&matrix, pph, nonce);
    let pow_val = Uint256::from_be_bytes(pow_hash.as_bytes());
    pow_val <= target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_helpers() {
        let target_arr = wasm_calc_target(0x1d00ffff);
        assert!(wasm_is_valid_target(&target_arr));
    }
}
