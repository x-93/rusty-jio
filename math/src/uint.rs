#![allow(clippy::needless_range_loop)]

use std::cmp::Ordering;
use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not,
    Shl, Shr, Sub, SubAssign,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A 192-bit unsigned integer stored as three 64-bit limbs in little-endian order.
/// Commonly used for cumulative blue work in GHOSTDAG consensus.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Uint192(pub [u64; 3]);

impl Uint192 {
    pub const ZERO: Self = Self([0, 0, 0]);
    pub const ONE: Self = Self([1, 0, 0]);
    pub const MAX: Self = Self([u64::MAX, u64::MAX, u64::MAX]);

    /// Create from a single u64 value.
    pub const fn from_u64(val: u64) -> Self {
        Self([val, 0, 0])
    }

    /// Create from a single u128 value.
    pub const fn from_u128(val: u128) -> Self {
        Self([val as u64, (val >> 64) as u64, 0])
    }

    /// Return the lowest 64 bits as a u64.
    pub const fn as_u64(&self) -> u64 {
        self.0[0]
    }

    /// True if value is zero.
    pub const fn is_zero(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0
    }

    /// Create a Uint192 from 24 raw bytes (Little-Endian).
    pub fn from_le_bytes(bytes: [u8; 24]) -> Self {
        let mut limbs = [0u64; 3];
        for i in 0..3 {
            let start = i * 8;
            let chunk: [u8; 8] = bytes[start..start + 8].try_into().unwrap();
            limbs[i] = u64::from_le_bytes(chunk);
        }
        Self(limbs)
    }

    /// Try to create a Uint192 from a little-endian byte slice.
    pub fn from_le_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 24 {
            return Err("Slice length must be exactly 24 bytes for Uint192");
        }
        let arr: [u8; 24] = slice.try_into().unwrap();
        Ok(Self::from_le_bytes(arr))
    }

    /// Convert Uint192 into 24 raw bytes (Little-Endian).
    pub fn to_le_bytes(self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        for i in 0..3 {
            let start = i * 8;
            bytes[start..start + 8].copy_from_slice(&self.0[i].to_le_bytes());
        }
        bytes
    }

    /// Create a Uint192 from 24 raw bytes (Big-Endian).
    pub fn from_be_bytes(bytes: [u8; 24]) -> Self {
        let mut limbs = [0u64; 3];
        for i in 0..3 {
            let start = (2 - i) * 8;
            let chunk: [u8; 8] = bytes[start..start + 8].try_into().unwrap();
            limbs[i] = u64::from_be_bytes(chunk);
        }
        Self(limbs)
    }

    /// Try to create a Uint192 from a big-endian byte slice.
    pub fn from_be_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 24 {
            return Err("Slice length must be exactly 24 bytes for Uint192");
        }
        let arr: [u8; 24] = slice.try_into().unwrap();
        Ok(Self::from_be_bytes(arr))
    }

    /// Convert Uint192 into 24 raw bytes (Big-Endian).
    pub fn to_be_bytes(self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        for i in 0..3 {
            let start = (2 - i) * 8;
            bytes[start..start + 8].copy_from_slice(&self.0[i].to_be_bytes());
        }
        bytes
    }

    /// Add two Uint192 numbers, returning (result, overflow_occurred).
    pub fn overflowing_add(self, other: Self) -> (Self, bool) {
        let mut result = [0u64; 3];
        let mut carry = false;

        for i in 0..3 {
            let (sum1, ov1) = self.0[i].overflowing_add(other.0[i]);
            let (sum2, ov2) = sum1.overflowing_add(u64::from(carry));
            result[i] = sum2;
            carry = ov1 || ov2;
        }

        (Self(result), carry)
    }

    /// Subtract two Uint192 numbers, returning (result, underflow_occurred).
    pub fn overflowing_sub(self, other: Self) -> (Self, bool) {
        let mut result = [0u64; 3];
        let mut borrow = false;

        for i in 0..3 {
            let (diff1, un1) = self.0[i].overflowing_sub(other.0[i]);
            let (diff2, un2) = diff1.overflowing_sub(u64::from(borrow));
            result[i] = diff2;
            borrow = un1 || un2;
        }

        (Self(result), borrow)
    }

    /// Multiply two Uint192 numbers, returning (result, overflow_occurred).
    pub fn overflowing_mul(self, other: Self) -> (Self, bool) {
        let mut res = [0u64; 3];
        let mut overflow = false;

        for i in 0..3 {
            let mut carry = 0u128;
            for j in 0..3 {
                let k = i + j;
                let prod = (self.0[i] as u128) * (other.0[j] as u128) + carry;
                if k < 3 {
                    let total = (res[k] as u128) + (prod as u64 as u128);
                    res[k] = total as u64;
                    carry = (prod >> 64) + (total >> 64);
                } else if prod != 0 {
                    overflow = true;
                }
            }
            if carry != 0 {
                overflow = true;
            }
        }

        (Self(res), overflow)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        let (res, ov) = self.overflowing_add(other);
        if ov { None } else { Some(res) }
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        let (res, un) = self.overflowing_sub(other);
        if un { None } else { Some(res) }
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let (res, ov) = self.overflowing_mul(other);
        if ov { None } else { Some(res) }
    }

    pub fn saturating_add(self, other: Self) -> Self {
        let (res, ov) = self.overflowing_add(other);
        if ov { Self::MAX } else { res }
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        let (res, un) = self.overflowing_sub(other);
        if un { Self::ZERO } else { res }
    }

    /// Total number of bits required to represent the number.
    pub fn bits(&self) -> usize {
        for i in (0..3).rev() {
            if self.0[i] != 0 {
                return (i + 1) * 64 - self.0[i].leading_zeros() as usize;
            }
        }
        0
    }
}

impl Ord for Uint192 {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..3).rev() {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Uint192 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Uint192 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let (res, ov) = self.overflowing_add(other);
        assert!(!ov, "Uint192 addition overflowed");
        res
    }
}

impl AddAssign for Uint192 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Uint192 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let (res, un) = self.overflowing_sub(other);
        assert!(!un, "Uint192 subtraction underflowed");
        res
    }
}

impl SubAssign for Uint192 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul for Uint192 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let (res, ov) = self.overflowing_mul(other);
        assert!(!ov, "Uint192 multiplication overflowed");
        res
    }
}

impl BitAnd for Uint192 {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
        ])
    }
}

impl BitAndAssign for Uint192 {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

impl BitOr for Uint192 {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
        ])
    }
}

impl BitOrAssign for Uint192 {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitXor for Uint192 {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        Self([
            self.0[0] ^ other.0[0],
            self.0[1] ^ other.0[1],
            self.0[2] ^ other.0[2],
        ])
    }
}

impl BitXorAssign for Uint192 {
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

impl Not for Uint192 {
    type Output = Self;
    fn not(self) -> Self {
        Self([!self.0[0], !self.0[1], !self.0[2]])
    }
}

impl Shl<usize> for Uint192 {
    type Output = Self;
    fn shl(self, shift: usize) -> Self {
        if shift >= 192 {
            return Self::ZERO;
        }
        let limb_shift = shift / 64;
        let bit_shift = shift % 64;
        let mut limbs = [0u64; 3];

        for i in 0..3 {
            if i + limb_shift < 3 {
                limbs[i + limb_shift] |= self.0[i] << bit_shift;
            }
            if bit_shift > 0 && i + limb_shift + 1 < 3 {
                limbs[i + limb_shift + 1] |= self.0[i] >> (64 - bit_shift);
            }
        }
        Self(limbs)
    }
}

impl Shr<usize> for Uint192 {
    type Output = Self;
    fn shr(self, shift: usize) -> Self {
        if shift >= 192 {
            return Self::ZERO;
        }
        let limb_shift = shift / 64;
        let bit_shift = shift % 64;
        let mut limbs = [0u64; 3];

        for i in 0..3 {
            if i >= limb_shift {
                limbs[i - limb_shift] |= self.0[i] >> bit_shift;
            }
            if bit_shift > 0 && i > limb_shift {
                limbs[i - limb_shift - 1] |= self.0[i] << (64 - bit_shift);
            }
        }
        Self(limbs)
    }
}

impl From<u64> for Uint192 {
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u128> for Uint192 {
    fn from(val: u128) -> Self {
        Self::from_u128(val)
    }
}

impl From<[u64; 3]> for Uint192 {
    fn from(limbs: [u64; 3]) -> Self {
        Self(limbs)
    }
}

impl fmt::Display for Uint192 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}{:016x}{:016x}", self.0[2], self.0[1], self.0[0])
    }
}

impl fmt::LowerHex for Uint192 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}{:016x}{:016x}", self.0[2], self.0[1], self.0[0])
    }
}

impl fmt::UpperHex for Uint192 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016X}{:016X}{:016X}", self.0[2], self.0[1], self.0[0])
    }
}

impl Serialize for Uint192 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("{:x}", self))
        } else {
            self.to_le_bytes().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Uint192 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = hex_decode_192(&s).map_err(serde::de::Error::custom)?;
            Ok(Self::from_be_bytes(bytes))
        } else {
            let bytes = <[u8; 24]>::deserialize(deserializer)?;
            Ok(Self::from_le_bytes(bytes))
        }
    }
}

/// A 256-bit unsigned integer stored as four 64-bit limbs in little-endian order.
/// Commonly used for difficulty targets, hashes, and large arithmetic.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Uint256(pub [u64; 4]);

impl Uint256 {
    pub const ZERO: Self = Self([0, 0, 0, 0]);
    pub const ONE: Self = Self([1, 0, 0, 0]);
    pub const MAX: Self = Self([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);

    /// Create from a single u64 value.
    pub const fn from_u64(val: u64) -> Self {
        Self([val, 0, 0, 0])
    }

    /// Create from a single u128 value.
    pub const fn from_u128(val: u128) -> Self {
        Self([val as u64, (val >> 64) as u64, 0, 0])
    }

    /// Return the lowest 64 bits as a u64.
    pub const fn as_u64(&self) -> u64 {
        self.0[0]
    }

    /// True if value is zero.
    pub const fn is_zero(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    /// Create a Uint256 from 32 raw bytes (Little-Endian).
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let start = i * 8;
            let chunk: [u8; 8] = bytes[start..start + 8].try_into().unwrap();
            limbs[i] = u64::from_le_bytes(chunk);
        }
        Self(limbs)
    }

    /// Try to create a Uint256 from a little-endian byte slice.
    pub fn from_le_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 32 {
            return Err("Slice length must be exactly 32 bytes for Uint256");
        }
        let arr: [u8; 32] = slice.try_into().unwrap();
        Ok(Self::from_le_bytes(arr))
    }

    /// Convert Uint256 into 32 raw bytes (Little-Endian).
    pub fn to_le_bytes(self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            let start = i * 8;
            bytes[start..start + 8].copy_from_slice(&self.0[i].to_le_bytes());
        }
        bytes
    }

    /// Create a Uint256 from 32 raw bytes (Big-Endian).
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0u64; 4];
        for i in 0..4 {
            let start = (3 - i) * 8;
            let chunk: [u8; 8] = bytes[start..start + 8].try_into().unwrap();
            limbs[i] = u64::from_be_bytes(chunk);
        }
        Self(limbs)
    }

    /// Try to create a Uint256 from a big-endian byte slice.
    pub fn from_be_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 32 {
            return Err("Slice length must be exactly 32 bytes for Uint256");
        }
        let arr: [u8; 32] = slice.try_into().unwrap();
        Ok(Self::from_be_bytes(arr))
    }

    /// Convert Uint256 into 32 raw bytes (Big-Endian).
    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        for i in 0..4 {
            let start = (3 - i) * 8;
            bytes[start..start + 8].copy_from_slice(&self.0[i].to_be_bytes());
        }
        bytes
    }

    /// Add two Uint256 numbers, returning (result, overflow_occurred).
    pub fn overflowing_add(self, other: Self) -> (Self, bool) {
        let mut result = [0u64; 4];
        let mut carry = false;

        for i in 0..4 {
            let (sum1, ov1) = self.0[i].overflowing_add(other.0[i]);
            let (sum2, ov2) = sum1.overflowing_add(u64::from(carry));
            result[i] = sum2;
            carry = ov1 || ov2;
        }

        (Self(result), carry)
    }

    /// Subtract two Uint256 numbers, returning (result, underflow_occurred).
    pub fn overflowing_sub(self, other: Self) -> (Self, bool) {
        let mut result = [0u64; 4];
        let mut borrow = false;

        for i in 0..4 {
            let (diff1, un1) = self.0[i].overflowing_sub(other.0[i]);
            let (diff2, un2) = diff1.overflowing_sub(u64::from(borrow));
            result[i] = diff2;
            borrow = un1 || un2;
        }

        (Self(result), borrow)
    }

    /// Multiply two Uint256 numbers, returning (result, overflow_occurred).
    pub fn overflowing_mul(self, other: Self) -> (Self, bool) {
        let mut res = [0u64; 4];
        let mut overflow = false;

        for i in 0..4 {
            let mut carry = 0u128;
            for j in 0..4 {
                let k = i + j;
                let prod = (self.0[i] as u128) * (other.0[j] as u128) + carry;
                if k < 4 {
                    let total = (res[k] as u128) + (prod as u64 as u128);
                    res[k] = total as u64;
                    carry = (prod >> 64) + (total >> 64);
                } else if prod != 0 {
                    overflow = true;
                }
            }
            if carry != 0 {
                overflow = true;
            }
        }

        (Self(res), overflow)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        let (res, ov) = self.overflowing_add(other);
        if ov { None } else { Some(res) }
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        let (res, un) = self.overflowing_sub(other);
        if un { None } else { Some(res) }
    }

    pub fn checked_mul(self, other: Self) -> Option<Self> {
        let (res, ov) = self.overflowing_mul(other);
        if ov { None } else { Some(res) }
    }

    pub fn saturating_add(self, other: Self) -> Self {
        let (res, ov) = self.overflowing_add(other);
        if ov { Self::MAX } else { res }
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        let (res, un) = self.overflowing_sub(other);
        if un { Self::ZERO } else { res }
    }

    /// Total number of bits required to represent the number.
    pub fn bits(&self) -> usize {
        for i in (0..4).rev() {
            if self.0[i] != 0 {
                return (i + 1) * 64 - self.0[i].leading_zeros() as usize;
            }
        }
        0
    }

    /// Decode compact target "bits" (stored in block headers) into a full 256-bit PoW target.
    pub fn from_compact_target_bits(bits: u32) -> Self {
        let size = (bits >> 24) as usize;
        let mantissa = (bits & 0x007f_ffff) as u64;
        if size <= 3 {
            Self::from_u64(mantissa >> (8 * (3 - size)))
        } else {
            let shift = 8 * (size - 3);
            let limb_index = shift / 64;
            let bit_offset = shift % 64;
            let mut limbs = [0u64; 4];
            if limb_index < 4 {
                limbs[limb_index] = mantissa << bit_offset;
                if bit_offset > 0 && limb_index + 1 < 4 {
                    limbs[limb_index + 1] = mantissa >> (64 - bit_offset);
                }
            }
            Self(limbs)
        }
    }

    /// Encode a 256-bit PoW target into compact format (32-bit compact target).
    pub fn to_compact_target_bits(&self) -> u32 {
        let mut size = self.bits().div_ceil(8);
        let mut compact = if size <= 3 {
            (self.as_u64() << (8 * (3 - size))) as u32
        } else {
            let shifted = *self >> (8 * (size - 3));
            shifted.as_u64() as u32
        };

        if compact & 0x0080_0000 != 0 {
            compact >>= 8;
            size += 1;
        }

        compact | ((size as u32) << 24)
    }
}

impl Ord for Uint256 {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..4).rev() {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Equal => continue,
                non_eq => return non_eq,
            }
        }
        Ordering::Equal
    }
}

impl PartialOrd for Uint256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Uint256 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let (res, ov) = self.overflowing_add(other);
        assert!(!ov, "Uint256 addition overflowed");
        res
    }
}

impl AddAssign for Uint256 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Uint256 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let (res, un) = self.overflowing_sub(other);
        assert!(!un, "Uint256 subtraction underflowed");
        res
    }
}

impl SubAssign for Uint256 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul for Uint256 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let (res, ov) = self.overflowing_mul(other);
        assert!(!ov, "Uint256 multiplication overflowed");
        res
    }
}

impl BitAnd for Uint256 {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
            self.0[3] & other.0[3],
        ])
    }
}

impl BitAndAssign for Uint256 {
    fn bitand_assign(&mut self, other: Self) {
        *self = *self & other;
    }
}

impl BitOr for Uint256 {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
            self.0[3] | other.0[3],
        ])
    }
}

impl BitOrAssign for Uint256 {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

impl BitXor for Uint256 {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        Self([
            self.0[0] ^ other.0[0],
            self.0[1] ^ other.0[1],
            self.0[2] ^ other.0[2],
            self.0[3] ^ other.0[3],
        ])
    }
}

impl BitXorAssign for Uint256 {
    fn bitxor_assign(&mut self, other: Self) {
        *self = *self ^ other;
    }
}

impl Not for Uint256 {
    type Output = Self;
    fn not(self) -> Self {
        Self([!self.0[0], !self.0[1], !self.0[2], !self.0[3]])
    }
}

impl Shl<usize> for Uint256 {
    type Output = Self;
    fn shl(self, shift: usize) -> Self {
        if shift >= 256 {
            return Self::ZERO;
        }
        let limb_shift = shift / 64;
        let bit_shift = shift % 64;
        let mut limbs = [0u64; 4];

        for i in 0..4 {
            if i + limb_shift < 4 {
                limbs[i + limb_shift] |= self.0[i] << bit_shift;
            }
            if bit_shift > 0 && i + limb_shift + 1 < 4 {
                limbs[i + limb_shift + 1] |= self.0[i] >> (64 - bit_shift);
            }
        }
        Self(limbs)
    }
}

impl Shr<usize> for Uint256 {
    type Output = Self;
    fn shr(self, shift: usize) -> Self {
        if shift >= 256 {
            return Self::ZERO;
        }
        let limb_shift = shift / 64;
        let bit_shift = shift % 64;
        let mut limbs = [0u64; 4];

        for i in 0..4 {
            if i >= limb_shift {
                limbs[i - limb_shift] |= self.0[i] >> bit_shift;
            }
            if bit_shift > 0 && i > limb_shift {
                limbs[i - limb_shift - 1] |= self.0[i] << (64 - bit_shift);
            }
        }
        Self(limbs)
    }
}

impl From<u64> for Uint256 {
    fn from(val: u64) -> Self {
        Self::from_u64(val)
    }
}

impl From<u128> for Uint256 {
    fn from(val: u128) -> Self {
        Self::from_u128(val)
    }
}

impl From<[u64; 4]> for Uint256 {
    fn from(limbs: [u64; 4]) -> Self {
        Self(limbs)
    }
}

/// Widening conversion from Uint192 to Uint256 is always lossless.
impl From<Uint192> for Uint256 {
    fn from(val: Uint192) -> Self {
        Self([val.0[0], val.0[1], val.0[2], 0])
    }
}

/// Narrowing conversion from Uint256 to Uint192.
impl TryFrom<Uint256> for Uint192 {
    type Error = &'static str;

    fn try_from(val: Uint256) -> Result<Self, Self::Error> {
        if val.0[3] != 0 {
            Err("Uint256 exceeds 192-bit capacity")
        } else {
            Ok(Self([val.0[0], val.0[1], val.0[2]]))
        }
    }
}

impl fmt::Display for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:016x}{:016x}{:016x}{:016x}",
            self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}

impl fmt::LowerHex for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:016x}{:016x}{:016x}{:016x}",
            self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}

impl fmt::UpperHex for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:016X}{:016X}{:016X}{:016X}",
            self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}

impl Serialize for Uint256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&format!("{:x}", self))
        } else {
            self.to_le_bytes().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Uint256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = hex_decode_256(&s).map_err(serde::de::Error::custom)?;
            Ok(Self::from_be_bytes(bytes))
        } else {
            let bytes = <[u8; 32]>::deserialize(deserializer)?;
            Ok(Self::from_le_bytes(bytes))
        }
    }
}

fn hex_decode_192(hex: &str) -> Result<[u8; 24], String> {
    let hex = hex.trim();
    let hex = hex.strip_prefix("0x").or_else(|| hex.strip_prefix("0X")).unwrap_or(hex);
    let hex_trimmed = hex.trim_start_matches('0');
    let hex = if hex_trimmed.is_empty() { "0" } else { hex_trimmed };
    if hex.len() > 48 {
        return Err(format!("Hex string too long for Uint192: {}", hex.len()));
    }
    let padded = format!("{:0>48}", hex);
    let mut out = [0u8; 24];
    for (i, chunk) in padded.as_bytes().chunks(2).enumerate() {
        let hi = parse_nibble(chunk[0])?;
        let lo = parse_nibble(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn hex_decode_256(hex: &str) -> Result<[u8; 32], String> {
    let hex = hex.trim();
    let hex = hex.strip_prefix("0x").or_else(|| hex.strip_prefix("0X")).unwrap_or(hex);
    let hex_trimmed = hex.trim_start_matches('0');
    let hex = if hex_trimmed.is_empty() { "0" } else { hex_trimmed };
    if hex.len() > 64 {
        return Err(format!("Hex string too long for Uint256: {}", hex.len()));
    }
    let padded = format!("{:0>64}", hex);
    let mut out = [0u8; 32];
    for (i, chunk) in padded.as_bytes().chunks(2).enumerate() {
        let hi = parse_nibble(chunk[0])?;
        let lo = parse_nibble(chunk[1])?;
        out[i] = (hi << 4) | lo;
    }
    Ok(out)
}

fn parse_nibble(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("Invalid hex character: {}", b as char)),
    }
}
