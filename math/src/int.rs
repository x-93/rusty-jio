use crate::uint::{Uint128, Uint192, Uint256};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    Overflow,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => write!(f, "value exceeds destination integer bit width"),
        }
    }
}

impl std::error::Error for ConversionError {}

// --- From conversions (Lossless / Widening) ---

impl From<u64> for Uint256 {
    #[inline(always)]
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u128> for Uint256 {
    #[inline(always)]
    fn from(val: u128) -> Self {
        Self::from_u128(val)
    }
}

impl From<Uint128> for Uint256 {
    #[inline(always)]
    fn from(val: Uint128) -> Self {
        Self([val.0[0], val.0[1], 0, 0])
    }
}

impl From<Uint192> for Uint256 {
    #[inline(always)]
    fn from(val: Uint192) -> Self {
        Self([val.0[0], val.0[1], val.0[2], 0])
    }
}

impl From<u64> for Uint128 {
    #[inline(always)]
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u32> for Uint128 {
    #[inline(always)]
    fn from(val: u32) -> Self {
        Self::from_u64(val as u64)
    }
}

impl From<i32> for Uint128 {
    #[inline(always)]
    fn from(val: i32) -> Self {
        Self::from_u64(val as u64)
    }
}

impl From<u64> for Uint192 {
    #[inline(always)]
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u32> for Uint192 {
    #[inline(always)]
    fn from(val: u32) -> Self {
        Self::from_u64(val as u64)
    }
}

impl From<i32> for Uint192 {
    #[inline(always)]
    fn from(val: i32) -> Self {
        Self::from_u64(val as u64)
    }
}

impl From<u32> for Uint256 {
    #[inline(always)]
    fn from(val: u32) -> Self {
        Self::from_u64(val as u64)
    }
}

impl From<i32> for Uint256 {
    #[inline(always)]
    fn from(val: i32) -> Self {
        Self::from_u64(val as u64)
    }
}

// --- TryFrom conversions (Narrowing with Overflow Detection) ---

impl TryFrom<Uint256> for u64 {
    type Error = ConversionError;
    fn try_from(val: Uint256) -> Result<Self, Self::Error> {
        if val.0[1] != 0 || val.0[2] != 0 || val.0[3] != 0 {
            return Err(ConversionError::Overflow);
        }
        Ok(val.0[0])
    }
}

impl TryFrom<Uint256> for u128 {
    type Error = ConversionError;
    fn try_from(val: Uint256) -> Result<Self, Self::Error> {
        if val.0[2] != 0 || val.0[3] != 0 {
            return Err(ConversionError::Overflow);
        }
        Ok((val.0[0] as u128) | ((val.0[1] as u128) << 64))
    }
}

impl TryFrom<Uint256> for Uint128 {
    type Error = ConversionError;
    fn try_from(val: Uint256) -> Result<Self, Self::Error> {
        if val.0[2] != 0 || val.0[3] != 0 {
            return Err(ConversionError::Overflow);
        }
        Ok(Uint128([val.0[0], val.0[1]]))
    }
}

impl TryFrom<Uint256> for Uint192 {
    type Error = ConversionError;
    fn try_from(val: Uint256) -> Result<Self, Self::Error> {
        if val.0[3] != 0 {
            return Err(ConversionError::Overflow);
        }
        Ok(Uint192([val.0[0], val.0[1], val.0[2]]))
    }
}

// --- Floating-point conversion for DAA / Difficulty Calculations ---

impl Uint256 {
    /// Convert wide integer to f64 approximation for difficulty & target calculations
    pub fn to_f64(&self) -> f64 {
        let mut factor = 1.0f64;
        let mut result = 0.0f64;
        for &limb in &self.0 {
            result += (limb as f64) * factor;
            factor *= 18446744073709551616.0f64; // 2^64
        }
        result
    }

    /// Construct from non-negative f64 approximation
    pub fn from_f64_saturating(val: f64) -> Self {
        if val <= 0.0 || val.is_nan() {
            return Self::ZERO;
        }
        if val >= f64::MAX || val.is_infinite() {
            return Self::MAX;
        }

        let mut res = Self::ZERO;
        let mut rem = val;
        let base = 18446744073709551616.0f64; // 2^64

        let mut i = 0;
        while i < 4 && rem > 0.0 {
            let limb = (rem % base) as u64;
            res.0[i] = limb;
            rem = (rem / base).floor();
            i += 1;
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversions_widening_and_narrowing() {
        let u_val = 42u64;
        let wide: Uint256 = u_val.into();
        assert_eq!(wide.0, [42, 0, 0, 0]);

        let narrow: u64 = wide.try_into().unwrap();
        assert_eq!(narrow, 42);

        let overflow_val = Uint256::from_limbs([0, 1, 0, 0]);
        let err: Result<u64, _> = overflow_val.try_into();
        assert_eq!(err, Err(ConversionError::Overflow));
    }

    #[test]
    fn test_f64_conversion() {
        let val = Uint256::from_u64(1_000_000);
        assert!((val.to_f64() - 1_000_000.0).abs() < 1e-5);
    }
}
