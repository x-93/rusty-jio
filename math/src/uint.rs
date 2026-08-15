use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};

macro_rules! construct_uint {
    ($name:ident, $n_words:expr) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
        pub struct $name(pub [u64; $n_words]);

        impl $name {
            pub const ZERO: Self = Self([0u64; $n_words]);
            pub const ONE: Self = {
                let mut arr = [0u64; $n_words];
                arr[0] = 1;
                Self(arr)
            };
            pub const MAX: Self = Self([u64::MAX; $n_words]);
            pub const BYTES: usize = $n_words * 8;
            pub const BITS: usize = $n_words * 64;

            #[inline]
            pub const fn from_u64(val: u64) -> Self {
                let mut arr = [0u64; $n_words];
                arr[0] = val;
                Self(arr)
            }

            #[inline]
            pub const fn from_words(words: [u64; $n_words]) -> Self {
                Self(words)
            }

            #[inline]
            pub const fn as_words(&self) -> &[u64; $n_words] {
                &self.0
            }

            #[inline]
            pub const fn low_u64(&self) -> u64 {
                self.0[0]
            }

            #[inline]
            pub fn is_zero(&self) -> bool {
                self.0.iter().all(|&w| w == 0)
            }

            #[inline]
            pub fn wrapping_shl(self, shift: u32) -> Self {
                self << (shift as usize)
            }

            #[inline]
            pub fn wrapping_shr(self, shift: u32) -> Self {
                self >> (shift as usize)
            }

            #[inline]
            pub fn as_f64(&self) -> f64 {
                let mut res = 0.0f64;
                let mut base = 1.0f64;
                for &word in &self.0 {
                    res += (word as f64) * base;
                    base *= 18446744073709551616.0f64; // 2^64
                }
                res
            }

            pub fn bits(&self) -> usize {
                for (i, &w) in self.0.iter().enumerate().rev() {
                    if w != 0 {
                        return (i + 1) * 64 - w.leading_zeros() as usize;
                    }
                }
                0
            }

            pub fn to_le_bytes(&self) -> [u8; $n_words * 8] {
                let mut bytes = [0u8; $n_words * 8];
                for (i, &word) in self.0.iter().enumerate() {
                    bytes[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
                }
                bytes
            }

            pub fn to_be_bytes(&self) -> [u8; $n_words * 8] {
                let mut bytes = [0u8; $n_words * 8];
                for (i, &word) in self.0.iter().rev().enumerate() {
                    bytes[i * 8..(i + 1) * 8].copy_from_slice(&word.to_be_bytes());
                }
                bytes
            }

            pub fn from_le_bytes(bytes: [u8; $n_words * 8]) -> Self {
                let mut words = [0u64; $n_words];
                for (i, word) in words.iter_mut().enumerate() {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
                    *word = u64::from_le_bytes(b);
                }
                Self(words)
            }

            pub fn from_be_bytes(bytes: [u8; $n_words * 8]) -> Self {
                let mut words = [0u64; $n_words];
                for (i, word) in words.iter_mut().rev().enumerate() {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
                    *word = u64::from_be_bytes(b);
                }
                Self(words)
            }

            pub fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                let mut res = [0u64; $n_words];
                let mut carry = 0u64;
                for i in 0..$n_words {
                    let (sum1, c1) = self.0[i].overflowing_add(rhs.0[i]);
                    let (sum2, c2) = sum1.overflowing_add(carry);
                    res[i] = sum2;
                    carry = (c1 as u64) + (c2 as u64);
                }
                (Self(res), carry > 0)
            }

            pub fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                let mut res = [0u64; $n_words];
                let mut borrow = 0u64;
                for i in 0..$n_words {
                    let (diff1, b1) = self.0[i].overflowing_sub(rhs.0[i]);
                    let (diff2, b2) = diff1.overflowing_sub(borrow);
                    res[i] = diff2;
                    borrow = (b1 as u64) + (b2 as u64);
                }
                (Self(res), borrow > 0)
            }

            pub fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
                let mut res = [0u64; $n_words];
                let mut overflow = false;
                for i in 0..$n_words {
                    let mut carry = 0u128;
                    for j in 0..$n_words {
                        if i + j < $n_words {
                            let prod = (self.0[i] as u128) * (rhs.0[j] as u128)
                                + (res[i + j] as u128)
                                + carry;
                            res[i + j] = prod as u64;
                            carry = prod >> 64;
                        } else if self.0[i] != 0 && rhs.0[j] != 0 {
                            overflow = true;
                        }
                    }
                    if carry > 0 {
                        overflow = true;
                    }
                }
                (Self(res), overflow)
            }

            pub fn div_rem(self, rhs: Self) -> (Self, Self) {
                assert!(!rhs.is_zero(), "division by zero");
                if self < rhs {
                    return (Self::ZERO, self);
                }
                if self == rhs {
                    return (Self::ONE, Self::ZERO);
                }

                let mut quotient = Self::ZERO;
                let mut remainder = Self::ZERO;
                let num_bits = self.bits();

                for i in (0..num_bits).rev() {
                    remainder = remainder << 1;
                    if (self.0[i / 64] & (1 << (i % 64))) != 0 {
                        remainder.0[0] |= 1;
                    }

                    if remainder >= rhs {
                        remainder = remainder - rhs;
                        quotient.0[i / 64] |= 1 << (i % 64);
                    }
                }

                (quotient, remainder)
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                for i in (0..$n_words).rev() {
                    match self.0[i].cmp(&other.0[i]) {
                        Ordering::Equal => continue,
                        non_eq => return non_eq,
                    }
                }
                Ordering::Equal
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Add for $name {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                let (res, overflow) = self.overflowing_add(rhs);
                assert!(!overflow, "overflow in integer addition");
                res
            }
        }

        impl Sub for $name {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                let (res, underflow) = self.overflowing_sub(rhs);
                assert!(!underflow, "underflow in integer subtraction");
                res
            }
        }

        impl Mul for $name {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self {
                let (res, overflow) = self.overflowing_mul(rhs);
                assert!(!overflow, "overflow in integer multiplication");
                res
            }
        }

        impl Div for $name {
            type Output = Self;
            fn div(self, rhs: Self) -> Self {
                self.div_rem(rhs).0
            }
        }

        impl Rem for $name {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self {
                self.div_rem(rhs).1
            }
        }

        impl Shl<usize> for $name {
            type Output = Self;
            fn shl(self, shift: usize) -> Self {
                if shift >= $n_words * 64 {
                    return Self::ZERO;
                }
                let word_shift = shift / 64;
                let bit_shift = shift % 64;
                let mut res = [0u64; $n_words];

                for i in word_shift..$n_words {
                    res[i] = self.0[i - word_shift] << bit_shift;
                    if bit_shift > 0 && i > word_shift {
                        res[i] |= self.0[i - word_shift - 1] >> (64 - bit_shift);
                    }
                }
                Self(res)
            }
        }

        impl Shr<usize> for $name {
            type Output = Self;
            fn shr(self, shift: usize) -> Self {
                if shift >= $n_words * 64 {
                    return Self::ZERO;
                }
                let word_shift = shift / 64;
                let bit_shift = shift % 64;
                let mut res = [0u64; $n_words];

                for i in 0..($n_words - word_shift) {
                    res[i] = self.0[i + word_shift] >> bit_shift;
                    if bit_shift > 0 && i + word_shift + 1 < $n_words {
                        res[i] |= self.0[i + word_shift + 1] << (64 - bit_shift);
                    }
                }
                Self(res)
            }
        }

        impl BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                let mut res = [0u64; $n_words];
                for i in 0..$n_words {
                    res[i] = self.0[i] & rhs.0[i];
                }
                Self(res)
            }
        }

        impl BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                let mut res = [0u64; $n_words];
                for i in 0..$n_words {
                    res[i] = self.0[i] | rhs.0[i];
                }
                Self(res)
            }
        }

        impl BitXor for $name {
            type Output = Self;
            fn bitxor(self, rhs: Self) -> Self {
                let mut res = [0u64; $n_words];
                for i in 0..$n_words {
                    res[i] = self.0[i] ^ rhs.0[i];
                }
                Self(res)
            }
        }

        impl Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                let mut res = [0u64; $n_words];
                for i in 0..$n_words {
                    res[i] = !self.0[i];
                }
                Self(res)
            }
        }

        impl From<u8> for $name {
            fn from(v: u8) -> Self {
                Self::from_u64(v as u64)
            }
        }

        impl From<u16> for $name {
            fn from(v: u16) -> Self {
                Self::from_u64(v as u64)
            }
        }

        impl From<u32> for $name {
            fn from(v: u32) -> Self {
                Self::from_u64(v as u64)
            }
        }

        impl From<i32> for $name {
            fn from(v: i32) -> Self {
                assert!(v >= 0, "cannot convert negative integer to uint");
                Self::from_u64(v as u64)
            }
        }

        impl From<i64> for $name {
            fn from(v: i64) -> Self {
                assert!(v >= 0, "cannot convert negative integer to uint");
                Self::from_u64(v as u64)
            }
        }

        impl From<u64> for $name {
            fn from(v: u64) -> Self {
                Self::from_u64(v)
            }
        }

        impl From<u128> for $name {
            fn from(v: u128) -> Self {
                let mut arr = [0u64; $n_words];
                arr[0] = v as u64;
                if $n_words > 1 {
                    arr[1] = (v >> 64) as u64;
                }
                Self(arr)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let be = self.to_be_bytes();
                let hex_str = faster_hex::hex_string(&be);
                let trimmed = hex_str.trim_start_matches('0');
                if trimmed.is_empty() {
                    write!(f, "0")
                } else {
                    write!(f, "{trimmed}")
                }
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}(0x{})", stringify!($name), self.to_be_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>())
            }
        }

        impl fmt::LowerHex for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                for b in self.to_be_bytes() {
                    write!(f, "{b:02x}")?;
                }
                Ok(())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                if serializer.is_human_readable() {
                    let hex = format!("{:x}", self);
                    serializer.serialize_str(&hex)
                } else {
                    serializer.serialize_bytes(&self.to_le_bytes())
                }
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let s = String::deserialize(deserializer)?;
                    let mut bytes = [0u8; $n_words * 8];
                    let hex_s = if s.len() % 2 != 0 {
                        format!("0{s}")
                    } else {
                        s
                    };
                    let decoded = jio_utils::hex::decode_hex(&hex_s).map_err(serde::de::Error::custom)?;
                    if decoded.len() > bytes.len() {
                        return Err(serde::de::Error::custom("hex string too long for uint"));
                    }
                    let offset = bytes.len() - decoded.len();
                    bytes[offset..].copy_from_slice(&decoded);
                    Ok(Self::from_be_bytes(bytes))
                } else {
                    let bytes = <[u8; $n_words * 8]>::deserialize(deserializer)?;
                    Ok(Self::from_le_bytes(bytes))
                }
            }
        }

        impl borsh::BorshSerialize for $name {
            fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
                writer.write_all(&self.to_le_bytes())
            }
        }

        impl borsh::BorshDeserialize for $name {
            fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
                let mut bytes = [0u8; $n_words * 8];
                reader.read_exact(&mut bytes)?;
                Ok(Self::from_le_bytes(bytes))
            }
        }
    };
}

construct_uint!(Uint128, 2);
construct_uint!(Uint192, 3);
construct_uint!(Uint256, 4);

impl From<Uint128> for Uint256 {
    fn from(v: Uint128) -> Self {
        Self([v.0[0], v.0[1], 0, 0])
    }
}

impl From<Uint192> for Uint256 {
    fn from(v: Uint192) -> Self {
        Self([v.0[0], v.0[1], v.0[2], 0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uint256_arithmetic() {
        let a = Uint256::from(100u64);
        let b = Uint256::from(25u64);

        assert_eq!(a + b, Uint256::from(125u64));
        assert_eq!(a - b, Uint256::from(75u64));
        assert_eq!(a * b, Uint256::from(2500u64));
        assert_eq!(a / b, Uint256::from(4u64));
        assert_eq!(a % b, Uint256::ZERO);

        let (q, r) = a.div_rem(Uint256::from(30u64));
        assert_eq!(q, Uint256::from(3u64));
        assert_eq!(r, Uint256::from(10u64));
    }

    #[test]
    fn test_shifts() {
        let one = Uint256::ONE;
        assert_eq!(one << 1, Uint256::from(2u64));
        assert_eq!(one << 64, Uint256([0, 1, 0, 0]));
        assert_eq!(Uint256([0, 1, 0, 0]) >> 64, one);
    }

    #[test]
    fn test_endian_roundtrip() {
        let orig = Uint256([0x1122334455667788, 0x99aabbccddeeff00, 0x1234, 0x5678]);
        let le = orig.to_le_bytes();
        let from_le = Uint256::from_le_bytes(le);
        assert_eq!(orig, from_le);

        let be = orig.to_be_bytes();
        let from_be = Uint256::from_be_bytes(be);
        assert_eq!(orig, from_be);
    }
}
