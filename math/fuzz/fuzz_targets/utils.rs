//! Fuzzing helper utilities for comparing fixed-width uints against num-bigint.

use jio_math::{Uint128, Uint192, Uint256};
use num_bigint::BigUint;

pub fn uint128_to_biguint(val: Uint128) -> BigUint {
    BigUint::from_bytes_le(&val.to_le_bytes())
}

pub fn biguint_to_uint128(val: &BigUint) -> Uint128 {
    let bytes = val.to_bytes_le();
    let mut arr = [0u8; 16];
    let len = bytes.len().min(16);
    arr[..len].copy_from_slice(&bytes[..len]);
    Uint128::from_le_bytes(arr)
}

pub fn uint192_to_biguint(val: Uint192) -> BigUint {
    BigUint::from_bytes_le(&val.to_le_bytes())
}

pub fn biguint_to_uint192(val: &BigUint) -> Uint192 {
    let bytes = val.to_bytes_le();
    let mut arr = [0u8; 24];
    let len = bytes.len().min(24);
    arr[..len].copy_from_slice(&bytes[..len]);
    Uint192::from_le_bytes(arr)
}

pub fn uint256_to_biguint(val: Uint256) -> BigUint {
    BigUint::from_bytes_le(&val.to_le_bytes())
}

pub fn biguint_to_uint256(val: &BigUint) -> Uint256 {
    let bytes = val.to_bytes_le();
    let mut arr = [0u8; 32];
    let len = bytes.len().min(32);
    arr[..len].copy_from_slice(&bytes[..len]);
    Uint256::from_le_bytes(arr)
}

pub fn modulus_power_of_two(bits: usize) -> BigUint {
    BigUint::from(1u32) << bits
}