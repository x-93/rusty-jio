mod utils;

use jio_math::Uint128;
use num_bigint::BigUint;
use utils::{biguint_to_uint128, modulus_power_of_two, uint128_to_biguint};

pub fn verify_u128_ops(a: Uint128, b: Uint128) {
    let a_big = uint128_to_biguint(a);
    let b_big = uint128_to_biguint(b);
    let modulus = modulus_power_of_two(128);

    // Addition
    let (sum, carry) = a.overflowing_add(b);
    let sum_big = &a_big + &b_big;
    let expected_sum = &sum_big % &modulus;
    assert_eq!(uint128_to_biguint(sum), expected_sum);
    assert_eq!(carry, sum_big >= modulus);

    // Subtraction
    let (diff, borrow) = a.overflowing_sub(b);
    let expected_borrow = a_big < b_big;
    assert_eq!(borrow, expected_borrow);
    if !borrow {
        let expected_diff = &a_big - &b_big;
        assert_eq!(uint128_to_biguint(diff), expected_diff);
    } else {
        let expected_diff = (&a_big + &modulus) - &b_big;
        assert_eq!(uint128_to_biguint(diff), expected_diff);
    }

    // Multiplication
    let (prod, overflow) = a.overflowing_mul(b);
    let prod_big = &a_big * &b_big;
    let expected_prod = &prod_big % &modulus;
    assert_eq!(uint128_to_biguint(prod), expected_prod);
    assert_eq!(overflow, prod_big >= modulus);

    // Division & Remainder
    if !b.is_zero() {
        let (q, r) = a.div_rem(b).unwrap();
        let expected_q = &a_big / &b_big;
        let expected_r = &a_big % &b_big;
        assert_eq!(uint128_to_biguint(q), expected_q);
        assert_eq!(uint128_to_biguint(r), expected_r);
    }

    // Ordering
    assert_eq!(a.cmp(&b), a_big.cmp(&b_big));
    assert_eq!(a.ct_lt(&b).unwrap_u8() == 1, a_big < b_big);
    assert_eq!(a.ct_eq(&b).unwrap_u8() == 1, a_big == b_big);
}

fn main() {
    // Run sanity check against known edge cases
    let test_cases = [
        (Uint128::ZERO, Uint128::ZERO),
        (Uint128::ONE, Uint128::ONE),
        (Uint128::MAX, Uint128::ONE),
        (Uint128::MAX, Uint128::MAX),
        (Uint128::from(1234567890123456789u64), Uint128::from(9876543210987654321u64)),
        (Uint128::from([u64::MAX, 0]), Uint128::from([1, 0])),
    ];
    for (a, b) in test_cases {
        verify_u128_ops(a, b);
    }
    println!("u128 fuzz verification tests passed.");
}