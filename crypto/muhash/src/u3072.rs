//! 3072-bit modular arithmetic and rolling multiset hash implementation (MuHash3072).
//!
//! Uses the pseudo-Mersenne prime $p = 2^{3072} - 1103717$ for fast reduction.
//! Enables associative, commutative UTXO set commitment updates without tree rebalancing.

use jio_hashes::{Hash, MuHashElementHash, MuHashFinalizeHash};
use std::fmt::{self, Debug, Formatter};

pub const LIMBS: usize = 48; // 48 * 64 = 3072 bits
pub const BYTES: usize = 384; // 3072 / 8 = 384 bytes
pub const C: u64 = 1103717;

/// 3072-bit integer modulo $p = 2^{3072} - 1103717$.
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct Num3072(pub [u64; LIMBS]);

#[cfg(feature = "serde")]
impl serde::Serialize for Num3072 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(LIMBS))?;
        for limb in &self.0 {
            seq.serialize_element(limb)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Num3072 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Num3072Visitor;

        impl<'de> serde::de::Visitor<'de> for Num3072Visitor {
            type Value = Num3072;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of 48 u64 limbs")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut limbs = [0u64; LIMBS];
                for (i, limb) in limbs.iter_mut().enumerate() {
                    *limb = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(Num3072(limbs))
            }
        }

        deserializer.deserialize_seq(Num3072Visitor)
    }
}

impl Default for Num3072 {
    fn default() -> Self {
        Self::ONE
    }
}

impl Num3072 {
    pub const ZERO: Self = Self([0u64; LIMBS]);
    pub const ONE: Self = {
        let mut limbs = [0u64; LIMBS];
        limbs[0] = 1;
        Self(limbs)
    };

    pub fn from_u64(val: u64) -> Self {
        let mut limbs = [0u64; LIMBS];
        limbs[0] = val;
        Self(limbs)
    }

    pub fn to_bytes_le(&self) -> [u8; BYTES] {
        let mut bytes = [0u8; BYTES];
        for (i, &limb) in self.0.iter().enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&limb.to_le_bytes());
        }
        bytes
    }

    pub fn from_bytes_le(bytes: &[u8; BYTES]) -> Self {
        let mut limbs = [0u64; LIMBS];
        for i in 0..LIMBS {
            limbs[i] = u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into().unwrap());
        }
        let mut res = Self(limbs);
        res.normalize();
        res
    }

    /// Multiply two 3072-bit numbers modulo $p$.
    pub fn mul_mod(&self, rhs: &Self) -> Self {
        let mut wide = [0u64; LIMBS * 2];
        for i in 0..LIMBS {
            let mut carry = 0u128;
            for j in 0..LIMBS {
                let prod = (self.0[i] as u128) * (rhs.0[j] as u128) + (wide[i + j] as u128) + carry;
                wide[i + j] = prod as u64;
                carry = prod >> 64;
            }
            wide[i + LIMBS] = carry as u64;
        }

        Self(reduce_wide(wide))
    }

    /// Square self modulo $p$.
    pub fn square_mod(&self) -> Self {
        self.mul_mod(self)
    }

    /// Compute modular inverse via Fermat's Little Theorem: $a^{-1} \equiv a^{p-2} \pmod p$.
    pub fn inv_mod(&self) -> Self {
        let mut exp_limbs = [u64::MAX; LIMBS];
        exp_limbs[0] = u64::MAX - 1103718; // 2^3072 - 1103719

        let mut res = Self::ONE;
        let mut base = *self;

        for &limb in exp_limbs.iter() {
            let mut l = limb;
            for _ in 0..64 {
                if (l & 1) == 1 {
                    res = res.mul_mod(&base);
                }
                base = base.square_mod();
                l >>= 1;
            }
        }
        res
    }

    /// Normalize value so that it is strictly less than $p = 2^{3072} - C$.
    pub fn normalize(&mut self) {
        while gte_p(&self.0) {
            sub_p(&mut self.0);
        }
    }

    /// Map arbitrary byte slice into a uniform non-zero group element modulo $p$.
    pub fn from_data(data: &[u8]) -> Self {
        let mut bytes = [0u8; BYTES];
        let mut offset = 0;
        let mut nonce = 0u32;

        while offset < BYTES {
            let mut hasher = MuHashElementHash::new();
            hasher.update(data);
            hasher.update(nonce.to_le_bytes());
            let hash = hasher.finalize();
            let chunk_len = 32.min(BYTES - offset);
            bytes[offset..offset + chunk_len].copy_from_slice(&hash.as_bytes()[..chunk_len]);
            offset += chunk_len;
            nonce += 1;
        }

        let mut num = Self::from_bytes_le(&bytes);
        if num == Self::ZERO {
            num = Self::ONE;
        }
        num
    }
}

fn gte_p(limbs: &[u64; LIMBS]) -> bool {
    // p = 2^3072 - C, which is [u64::MAX - C + 1, u64::MAX, u64::MAX, ...]
    for i in (1..LIMBS).rev() {
        if limbs[i] < u64::MAX {
            return false;
        }
    }
    limbs[0] >= (u64::MAX - C + 1)
}

fn sub_p(limbs: &mut [u64; LIMBS]) {
    // Subtract p = add C and subtract 2^3072
    let mut carry = C as u128;
    for limb in limbs.iter_mut() {
        let sum = (*limb as u128) + carry;
        *limb = sum as u64;
        carry = sum >> 64;
    }
}

fn reduce_wide(mut wide: [u64; LIMBS * 2]) -> [u64; LIMBS] {
    let mut carry = 0u128;
    for i in 0..LIMBS {
        let prod = (wide[LIMBS + i] as u128) * (C as u128) + (wide[i] as u128) + carry;
        wide[i] = prod as u64;
        carry = prod >> 64;
    }

    while carry > 0 {
        let mut add_carry = carry * (C as u128);
        for item in wide.iter_mut().take(LIMBS) {
            let sum = (*item as u128) + add_carry;
            *item = sum as u64;
            add_carry = sum >> 64;
            if add_carry == 0 {
                break;
            }
        }
        carry = add_carry;
    }

    let mut out = [0u64; LIMBS];
    out.copy_from_slice(&wide[0..LIMBS]);
    while gte_p(&out) {
        sub_p(&mut out);
    }
    out
}

impl Debug for Num3072 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Num3072([..{} limbs])", LIMBS)
    }
}

/// Associative, rolling multiset hash for UTXO commitment updates.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct MuHash {
    acc: Num3072,
}

impl MuHash {
    /// Create a new empty MuHash accumulator (initialized to group identity 1).
    pub fn new() -> Self {
        Self { acc: Num3072::ONE }
    }

    /// Add an element to the multiset accumulator.
    pub fn add_element(&mut self, element: &[u8]) {
        let elem_num = Num3072::from_data(element);
        self.acc = self.acc.mul_mod(&elem_num);
    }

    /// Remove an element from the multiset accumulator by multiplying with its modular inverse.
    pub fn remove_element(&mut self, element: &[u8]) {
        let elem_num = Num3072::from_data(element);
        let inv = elem_num.inv_mod();
        self.acc = self.acc.mul_mod(&inv);
    }

    /// Combine another MuHash multiset accumulator into self.
    pub fn combine(&mut self, other: &Self) {
        self.acc = self.acc.mul_mod(&other.acc);
    }

    /// Finalize and compute the 32-byte cryptographic commitment hash.
    pub fn finalize(&self) -> Hash {
        let bytes = self.acc.to_bytes_le();
        let mut hasher = MuHashFinalizeHash::new();
        hasher.update(bytes);
        hasher.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muhash_associativity_and_removals() {
        let mut muhash1 = MuHash::new();
        let mut muhash2 = MuHash::new();

        let elem_a = b"txout_1:amount_50";
        let elem_b = b"txout_2:amount_100";
        let elem_c = b"txout_3:amount_25";

        // Add A and B to muhash1
        muhash1.add_element(elem_a);
        muhash1.add_element(elem_b);

        // Add B then A to muhash2 (order independence)
        muhash2.add_element(elem_b);
        muhash2.add_element(elem_a);

        assert_eq!(muhash1.finalize(), muhash2.finalize());

        // Add C then remove C
        muhash1.add_element(elem_c);
        assert_ne!(muhash1.finalize(), muhash2.finalize());
        muhash1.remove_element(elem_c);
        assert_eq!(muhash1.finalize(), muhash2.finalize());
    }
}
