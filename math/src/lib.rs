pub mod uint;
pub use uint::{Uint192, Uint256};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uint192_basic_math_and_comparison() {
        let a = Uint192::from_u64(u64::MAX);
        let b = Uint192::from_u64(1);

        // u64::MAX + 1 should spill over into limbs[1]
        let sum = a + b;
        assert_eq!(sum.0, [0, 1, 0]);
        assert!(sum > a);

        let diff = sum - b;
        assert_eq!(diff, a);

        // Underflow check
        let (underflow_res, un) = Uint192::ZERO.overflowing_sub(Uint192::ONE);
        assert!(un);
        assert_eq!(underflow_res, Uint192::MAX);

        // Multiplication
        let m1 = Uint192::from_u64(1_000_000);
        let m2 = Uint192::from_u64(2_000_000);
        assert_eq!(m1 * m2, Uint192::from_u64(2_000_000_000_000));
    }

    #[test]
    fn test_uint192_endianness_roundtrip() {
        let val = Uint192([0x0123456789abcdef, 0xfedcba9876543210, 0x1122334455667788]);
        let le = val.to_le_bytes();
        assert_eq!(Uint192::from_le_bytes(le), val);
        assert_eq!(Uint192::from_le_slice(&le).unwrap(), val);

        let be = val.to_be_bytes();
        assert_eq!(Uint192::from_be_bytes(be), val);
        assert_eq!(Uint192::from_be_slice(&be).unwrap(), val);
    }

    #[test]
    fn test_uint256_basic_math_and_comparison() {
        let a = Uint256::from_u64(u64::MAX);
        let b = Uint256::from_u64(1);

        let sum = a + b;
        assert_eq!(sum.0, [0, 1, 0, 0]);
        assert!(sum > a);

        let diff = sum - b;
        assert_eq!(diff, a);

        let (ov_res, ov) = Uint256::MAX.overflowing_add(Uint256::ONE);
        assert!(ov);
        assert_eq!(ov_res, Uint256::ZERO);
    }

    #[test]
    fn test_uint256_endianness_roundtrip() {
        let val = Uint256([
            0x0123456789abcdef,
            0xfedcba9876543210,
            0x1122334455667788,
            0x99aabbccddeeff00,
        ]);
        let le = val.to_le_bytes();
        assert_eq!(Uint256::from_le_bytes(le), val);
        assert_eq!(Uint256::from_le_slice(&le).unwrap(), val);

        let be = val.to_be_bytes();
        assert_eq!(Uint256::from_be_bytes(be), val);
        assert_eq!(Uint256::from_be_slice(&be).unwrap(), val);
    }

    #[test]
    fn test_compact_target_bits_roundtrip() {
        // Standard Bitcoin/Kaspa style compact bits
        let target_bits = 0x1d00ffff;
        let target = Uint256::from_compact_target_bits(target_bits);
        assert_eq!(target.to_compact_target_bits(), target_bits);

        let easy_bits = 0x207fffff;
        let easy_target = Uint256::from_compact_target_bits(easy_bits);
        assert_eq!(easy_target.to_compact_target_bits(), easy_bits);
    }

    #[test]
    fn test_uint192_uint256_conversion() {
        let u192 = Uint192([10, 20, 30]);
        let u256 = Uint256::from(u192);
        assert_eq!(u256.0, [10, 20, 30, 0]);

        let converted_back = Uint192::try_from(u256).unwrap();
        assert_eq!(converted_back, u192);

        let overflow_u256 = Uint256([10, 20, 30, 1]);
        assert!(Uint192::try_from(overflow_u256).is_err());
    }

    #[test]
    fn test_serde_json() {
        let val192 = Uint192([0x1234, 0x5678, 0x9abc]);
        let json192 = serde_json::to_string(&val192).unwrap();
        let de192: Uint192 = serde_json::from_str(&json192).unwrap();
        assert_eq!(val192, de192);

        let val256 = Uint256([0x1234, 0x5678, 0x9abc, 0xdef0]);
        let json256 = serde_json::to_string(&val256).unwrap();
        let de256: Uint256 = serde_json::from_str(&json256).unwrap();
        assert_eq!(val256, de256);

        // Test uppercase "0X" and leading zeros deserialization
        let custom_json = "\"0X00001234\"";
        let de_custom: Uint256 = serde_json::from_str(custom_json).unwrap();
        assert_eq!(de_custom, Uint256::from_u64(0x1234));
    }
}
