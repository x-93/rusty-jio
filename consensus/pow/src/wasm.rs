//! WASM bindings and helpers for PoW calculation.

use crate::calc_target;
use jio_math::Uint256;

pub fn wasm_calc_target(bits: u32) -> [u64; 4] {
    let target = calc_target(bits);
    target.0
}

pub fn wasm_is_valid_target(target: &[u64; 4]) -> bool {
    let t = Uint256(*target);
    t <= jio_consensus_core::config::constants::consensus::MAX_DIFFICULTY_TARGET
}