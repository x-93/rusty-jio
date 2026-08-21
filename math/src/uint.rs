use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr, Sub};

#[macro_export]
macro_rules! construct_uint {
    ($name:ident, $n_words:expr) => {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
        #[repr(C)]
        pub struct $name(pub [u64; $n_words]);

        impl $name {
            pub const BITS: usize = $n_words * 64;
            pub const BYTES: usize = $n_words * 8;
            pub const LIMBS: usize = $n_words;

            pub const ZERO: Self = Self([0; $n_words]);
            pub const ONE: Self = {
                let mut limbs = [0; $n_words];
                limbs[0] = 1;
                Self(limbs)
            };
            pub const MAX: Self = Self([u64::MAX; $n_words]);

            #[inline(always)]
            pub const fn from_limbs(limbs: [u64; $n_words]) -> Self {
                Self(limbs)
            }

            #[inline(always)]
            pub const fn as_limbs(&self) -> &[u64; $n_words] {
                &self.0
            }

            #[inline(always)]
            pub const fn from_u64(val: u64) -> Self {
                let mut limbs = [0; $n_words];
                limbs[0] = val;
                Self(limbs)
            }

            #[inline(always)]
            pub const fn from_u128(val: u128) -> Self {
                let mut limbs = [0; $n_words];
                limbs[0] = val as u64;
                if $n_words > 1 {
                    limbs[1] = (val >> 64) as u64;
                }
                Self(limbs)
            }

            #[inline(always)]
            pub fn is_zero(&self) -> bool {
                let mut acc = 0u64;
                let mut i = 0;
                while i < $n_words {
                    acc |= self.0[i];
                    i += 1;
                }
                acc == 0
            }

            /// Constant-time multi-limb addition with carry tracking.
            #[inline]
            pub fn carrying_add(self, rhs: Self) -> (Self, bool) {
                let mut ret = [0u64; $n_words];
                let mut carry = 0u128;

                let mut i = 0;
                while i < $n_words {
                    let sum = (self.0[i] as u128) + (rhs.0[i] as u128) + carry;
                    ret[i] = sum as u64;
                    carry = sum >> 64;
                    i += 1;
                }

                (Self(ret), carry != 0)
            }

            /// Constant-time multi-limb subtraction with borrow tracking.
            #[inline]
            pub fn borrowing_sub(self, rhs: Self) -> (Self, bool) {
                let mut ret = [0u64; $n_words];
                let mut borrow = 0u128;

                let mut i = 0;
                while i < $n_words {
                    let diff = (self.0[i] as u128)
                        .wrapping_sub(rhs.0[i] as u128)
                        .wrapping_sub(borrow);
                    ret[i] = diff as u64;
                    borrow = (diff >> 127) & 1;
                    i += 1;
                }

                (Self(ret), borrow != 0)
            }

            /// Multi-limb multiplication with carry accumulation and overflow checking.
            #[inline]
            pub fn carrying_mul(self, rhs: Self) -> (Self, bool) {
                let mut ret = [0u64; $n_words];
                let mut overflow = false;

                let mut i = 0;
                while i < $n_words {
                    let mut carry = 0u128;
                    let mut j = 0;
                    while j < $n_words {
                        let k = i + j;
                        let prod = (self.0[i] as u128) * (rhs.0[j] as u128) + carry;

                        if k < $n_words {
                            let total = (ret[k] as u128) + prod;
                            ret[k] = total as u64;
                            carry = total >> 64;
                        } else if prod != 0 {
                            overflow = true;
                        }
                        j += 1;
                    }
                    if carry != 0 {
                        overflow = true;
                    }
                    i += 1;
                }

                (Self(ret), overflow)
            }

            /// Multi-limb bitwise shift left.
            pub fn shl_unit(self, shift: u32) -> Self {
                if shift >= Self::BITS as u32 {
                    return Self::ZERO;
                }
                if shift == 0 {
                    return self;
                }

                let word_shift = (shift / 64) as usize;
                let bit_shift = shift % 64;
                let mut res = [0u64; $n_words];

                let mut i = $n_words;
                while i > word_shift {
                    i -= 1;
                    let src = i - word_shift;
                    let mut w = self.0[src] << bit_shift;
                    if bit_shift > 0 && src > 0 {
                        w |= self.0[src - 1] >> (64 - bit_shift);
                    }
                    res[i] = w;
                }

                Self(res)
            }

            /// Multi-limb bitwise shift right.
            pub fn shr_unit(self, shift: u32) -> Self {
                if shift >= Self::BITS as u32 {
                    return Self::ZERO;
                }
                if shift == 0 {
                    return self;
                }

                let word_shift = (shift / 64) as usize;
                let bit_shift = shift % 64;
                let mut res = [0u64; $n_words];

                let mut i = 0;
                while i + word_shift < $n_words {
                    let src = i + word_shift;
                    let mut w = self.0[src] >> bit_shift;
                    if bit_shift > 0 && src + 1 < $n_words {
                        w |= self.0[src + 1] << (64 - bit_shift);
                    }
                    res[i] = w;
                    i += 1;
                }

                Self(res)
            }

            /// Serialize into Little-Endian byte array.
            pub fn to_le_bytes(self) -> [u8; $n_words * 8] {
                let mut bytes = [0u8; $n_words * 8];
                let mut i = 0;
                while i < $n_words {
                    let chunk = self.0[i].to_le_bytes();
                    let offset = i * 8;
                    let mut j = 0;
                    while j < 8 {
                        bytes[offset + j] = chunk[j];
                        j += 1;
                    }
                    i += 1;
                }
                bytes
            }

            /// Deserialize from Little-Endian byte array.
            pub fn from_le_bytes(bytes: [u8; $n_words * 8]) -> Self {
                let mut limbs = [0u64; $n_words];
                let mut i = 0;
                while i < $n_words {
                    let offset = i * 8;
                    let mut chunk = [0u8; 8];
                    let mut j = 0;
                    while j < 8 {
                        chunk[j] = bytes[offset + j];
                        j += 1;
                    }
                    limbs[i] = u64::from_le_bytes(chunk);
                    i += 1;
                }
                Self(limbs)
            }

            /// Serialize into Big-Endian byte array.
            pub fn to_be_bytes(self) -> [u8; $n_words * 8] {
                let mut bytes = [0u8; $n_words * 8];
                let mut i = 0;
                while i < $n_words {
                    let chunk = self.0[$n_words - 1 - i].to_be_bytes();
                    let offset = i * 8;
                    let mut j = 0;
                    while j < 8 {
                        bytes[offset + j] = chunk[j];
                        j += 1;
                    }
                    i += 1;
                }
                bytes
            }

            /// Deserialize from Big-Endian byte array.
            pub fn from_be_bytes(bytes: [u8; $n_words * 8]) -> Self {
                let mut limbs = [0u64; $n_words];
                let mut i = 0;
                while i < $n_words {
                    let offset = i * 8;
                    let mut chunk = [0u8; 8];
                    let mut j = 0;
                    while j < 8 {
                        chunk[j] = bytes[offset + j];
                        j += 1;
                    }
                    limbs[$n_words - 1 - i] = u64::from_be_bytes(chunk);
                    i += 1;
                }
                Self(limbs)
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                let mut i = $n_words;
                while i > 0 {
                    i -= 1;
                    match self.0[i].cmp(&other.0[i]) {
                        Ordering::Equal => continue,
                        ordering => return ordering,
                    }
                }
                Ordering::Equal
            }
        }

        impl PartialOrd for $name {
            #[inline(always)]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Add for $name {
            type Output = Self;
            #[inline(always)]
            fn add(self, rhs: Self) -> Self {
                let (res, overflow) = self.carrying_add(rhs);
                debug_assert!(!overflow, "attempt to add with overflow");
                res
            }
        }

        impl Sub for $name {
            type Output = Self;
            #[inline(always)]
            fn sub(self, rhs: Self) -> Self {
                let (res, underflow) = self.borrowing_sub(rhs);
                debug_assert!(!underflow, "attempt to subtract with underflow");
                res
            }
        }

        impl Mul for $name {
            type Output = Self;
            #[inline(always)]
            fn mul(self, rhs: Self) -> Self {
                let (res, overflow) = self.carrying_mul(rhs);
                debug_assert!(!overflow, "attempt to multiply with overflow");
                res
            }
        }

        impl BitAnd for $name {
            type Output = Self;
            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self {
                let mut res = [0u64; $n_words];
                let mut i = 0;
                while i < $n_words {
                    res[i] = self.0[i] & rhs.0[i];
                    i += 1;
                }
                Self(res)
            }
        }

        impl BitOr for $name {
            type Output = Self;
            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self {
                let mut res = [0u64; $n_words];
                let mut i = 0;
                while i < $n_words {
                    res[i] = self.0[i] | rhs.0[i];
                    i += 1;
                }
                Self(res)
            }
        }

        impl BitXor for $name {
            type Output = Self;
            #[inline(always)]
            fn bitxor(self, rhs: Self) -> Self {
                let mut res = [0u64; $n_words];
                let mut i = 0;
                while i < $n_words {
                    res[i] = self.0[i] ^ rhs.0[i];
                    i += 1;
                }
                Self(res)
            }
        }

        impl Not for $name {
            type Output = Self;
            #[inline(always)]
            fn not(self) -> Self {
                let mut res = [0u64; $n_words];
                let mut i = 0;
                while i < $n_words {
                    res[i] = !self.0[i];
                    i += 1;
                }
                Self(res)
            }
        }

        impl Shl<u32> for $name {
            type Output = Self;
            #[inline(always)]
            fn shl(self, rhs: u32) -> Self {
                self.shl_unit(rhs)
            }
        }

        impl Shr<u32> for $name {
            type Output = Self;
            #[inline(always)]
            fn shr(self, rhs: u32) -> Self {
                self.shr_unit(rhs)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "0x")?;
                let mut i = $n_words;
                while i > 0 {
                    i -= 1;
                    write!(f, "{:016x}", self.0[i])?;
                }
                Ok(())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }

        impl fmt::LowerHex for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }
    };
}

construct_uint!(Uint128, 2);
construct_uint!(Uint192, 3);
construct_uint!(Uint256, 4);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carrying_add_basic_and_overflow() {
        let a = Uint256::from_limbs([u64::MAX, 0, 0, 0]);
        let b = Uint256::from_u64(1);
        let (res, overflow) = a.carrying_add(b);
        assert!(!overflow);
        assert_eq!(res.0, [0, 1, 0, 0]);

        let (max_res, max_overflow) = Uint256::MAX.carrying_add(Uint256::ONE);
        assert!(max_overflow);
        assert_eq!(max_res, Uint256::ZERO);
    }

    #[test]
    fn test_borrowing_sub_basic_and_underflow() {
        let a = Uint256::from_limbs([0, 1, 0, 0]);
        let b = Uint256::from_u64(1);
        let (res, underflow) = a.borrowing_sub(b);
        assert!(!underflow);
        assert_eq!(res.0, [u64::MAX, 0, 0, 0]);

        let (zero_res, zero_underflow) = Uint256::ZERO.borrowing_sub(Uint256::ONE);
        assert!(zero_underflow);
        assert_eq!(zero_res, Uint256::MAX);
    }

    #[test]
    fn test_carrying_mul() {
        let a = Uint256::from_limbs([u64::MAX, 0, 0, 0]);
        let b = Uint256::from_u64(2);
        let (res, overflow) = a.carrying_mul(b);
        assert!(!overflow);
        assert_eq!(res.0, [u64::MAX - 1, 1, 0, 0]);

        let high_val = Uint256::from_limbs([0, 0, 0, 1 << 63]);
        let (overflow_res, is_overflow) = high_val.carrying_mul(Uint256::from_u64(2));
        assert!(is_overflow);
        assert_eq!(overflow_res, Uint256::ZERO);
    }

    #[test]
    fn test_shifts() {
        let a = Uint256::ONE;
        assert_eq!(a.shl_unit(64).0, [0, 1, 0, 0]);
        assert_eq!(a.shl_unit(65).0, [0, 2, 0, 0]);
        assert_eq!(a.shl_unit(256), Uint256::ZERO);

        let b = Uint256::from_limbs([0, 2, 0, 0]);
        assert_eq!(b.shr_unit(65), Uint256::ONE);
        assert_eq!(b.shr_unit(256), Uint256::ZERO);
    }

    #[test]
    fn test_endian_serialization() {
        let val = Uint128::from_limbs([0x0123456789ABCDEF, 0x0FEDCBA987654321]);
        
        let le = val.to_le_bytes();
        let from_le = Uint128::from_le_bytes(le);
        assert_eq!(val, from_le);
        assert_eq!(le[0], 0xEF);
        assert_eq!(le[15], 0x0F);

        let be = val.to_be_bytes();
        let from_be = Uint128::from_be_bytes(be);
        assert_eq!(val, from_be);
        assert_eq!(be[0], 0x0F);
        assert_eq!(be[15], 0xEF);
    }

    #[test]
    fn test_ord_traits() {
        let low = Uint192::from_limbs([u64::MAX, u64::MAX, 0]);
        let high = Uint192::from_limbs([0, 0, 1]);
        assert!(high > low);
        assert_eq!(low.cmp(&low), Ordering::Equal);
    }
}