mod utils;

use jio_math::Uint192;
use num_bigint::BigUint;
use utils::{biguint_to_uint192, modulus_power_of_two, uint192_to_biguint};

pub fn verify_u192_ops(a: Uint192, b: Uint192) {
    let a_big = uint192_to_biguint(a);
    let b_big = uint192_to_biguint(b);
    let modulus = modulus_power_of_two(192);

    // Addition
    let (sum, carry) = a.overflowing_add(b);
    let sum_big = &a_big + &b_big;
    let expected_sum = &sum_big % &modulus;
    assert_eq!(uint192_to_biguint(sum), expected_sum);
    assert_eq!(carry, sum_big >= modulus);

    // Subtraction
    let (diff, borrow) = a.overflowing_sub(b);
    let expected_borrow = a_big < b_big;
    assert_eq!(borrow, expected_borrow);
    if !borrow {
        let expected_diff = &a_big - &b_big;
        assert_eq!(uint192_to_biguint(diff), expected_diff);
    } else {
        let expected_diff = (&a_big + &modulus) - &b_big;
        assert_eq!(uint192_to_biguint(diff), expected_diff);
    }

    // Multiplication
    let (prod, overflow) = a.overflowing_mul(b);
    let prod_big = &a_big * &b_big;
    let expected_prod = &prod_big % &modulus;
    assert_eq!(uint192_to_biguint(prod), expected_prod);
    assert_eq!(overflow, prod_big >= modulus);

    // Division & Remainder
    if !b.is_zero() {
        let (q, r) = a.div_rem(b).unwrap();
        let expected_q = &a_big / &b_big;
        let expected_r = &a_big % &b_big;
        assert_eq!(uint192_to_biguint(q), expected_q);
        assert_eq!(uint192_to_biguint(r), expected_r);
    }

    // Ordering
    assert_eq!(a.cmp(&b), a_big.cmp(&b_big));
    assert_eq!(a.ct_lt(&b).unwrap_u8() == 1, a_big < b_big);
    assert_eq!(a.ct_eq(&b).unwrap_u8() == 1, a_big == b_big);
}

fn main() {
    let test_cases = [
        (Uint192::ZERO, Uint192::ZERO),
        (Uint192::ONE, Uint192::ONE),
        (Uint192::MAX, Uint192::ONE),
        (Uint192::MAX, Uint192::MAX),
        (
            Uint192::from_limbs([u64::MAX, u64::MAX, 0]),
            Uint192::from_limbs([1, 0, 0]),
        ),
    ];
    for (a, b) in test_cases {
        verify_u192_ops(a, b);
    }
    println!("u192 fuzz verification tests passed.");
}