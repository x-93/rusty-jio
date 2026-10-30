pub mod hashers;
pub mod pow_hashers;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash as StdHash, Hasher as StdHasher};
use std::str::FromStr;

pub const HASH_SIZE: usize = 32;

pub use hashers::*;

/// A 32-byte cryptographic hash identifier used throughout the BlockDAG.
#[derive(Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub struct Hash(pub [u8; HASH_SIZE]);

impl Hash {
    pub const ZERO: Self = Self([0u8; HASH_SIZE]);

    #[inline(always)]
    pub const fn from_bytes(bytes: [u8; HASH_SIZE]) -> Self {
        Hash(bytes)
    }

    #[inline(always)]
    pub const fn as_bytes(&self) -> &[u8; HASH_SIZE] {
        &self.0
    }

    #[inline(always)]
    pub const fn to_bytes(self) -> [u8; HASH_SIZE] {
        self.0
    }

    #[inline(always)]
    pub fn from_slice(bytes: &[u8]) -> Self {
        Self(<[u8; HASH_SIZE]>::try_from(bytes).expect("Slice must have the length of Hash"))
    }

    #[inline(always)]
    pub fn to_le_u64(self) -> [u64; 4] {
        let mut out = [0u64; 4];
        out.iter_mut()
            .zip(self.iter_le_u64())
            .for_each(|(out, word)| *out = word);
        out
    }

    #[inline(always)]
    pub fn iter_le_u64(&self) -> impl ExactSizeIterator<Item = u64> + '_ {
        self.0
            .chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
    }

    #[inline(always)]
    pub fn from_le_u64(arr: [u64; 4]) -> Self {
        let mut ret = [0; HASH_SIZE];
        ret.chunks_exact_mut(8)
            .zip(arr.iter())
            .for_each(|(bytes, word)| bytes.copy_from_slice(&word.to_le_bytes()));
        Self(ret)
    }

    #[inline(always)]
    pub fn from_u64_word(word: u64) -> Self {
        Self::from_le_u64([0, 0, 0, word])
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

// Override the default Hash implementation for fast hashing (siphash over u64s)
impl StdHash for Hash {
    #[inline(always)]
    fn hash<H: StdHasher>(&self, state: &mut H) {
        self.iter_le_u64().for_each(|x| x.hash(state));
    }
}

impl PartialEq for Hash {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Display for Hash {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

impl FromStr for Hash {
    type Err = &'static str;

    #[inline]
    fn from_str(hash_str: &str) -> Result<Self, Self::Err> {
        let s = hash_str.trim();
        let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
        if s.len() != HASH_SIZE * 2 {
            return Err("Invalid hash hex string length");
        }
        let mut bytes = [0u8; HASH_SIZE];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hi = parse_hex_digit(chunk[0])?;
            let lo = parse_hex_digit(chunk[1])?;
            bytes[i] = (hi << 4) | lo;
        }
        Ok(Hash(bytes))
    }
}

impl From<u64> for Hash {
    #[inline(always)]
    fn from(word: u64) -> Self {
        Self::from_u64_word(word)
    }
}

impl From<[u8; HASH_SIZE]> for Hash {
    #[inline(always)]
    fn from(bytes: [u8; HASH_SIZE]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Hash> for [u8; HASH_SIZE] {
    #[inline(always)]
    fn from(hash: Hash) -> Self {
        hash.0
    }
}

impl AsRef<[u8]> for Hash {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Hash {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

pub const ZERO_HASH: Hash = Hash([0; HASH_SIZE]);

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            Self::from_str(&s).map_err(serde::de::Error::custom)
        } else {
            let bytes = <[u8; HASH_SIZE]>::deserialize(deserializer)?;
            Ok(Self(bytes))
        }
    }
}

fn parse_hex_digit(b: u8) -> Result<u8, &'static str> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err("Invalid hex character in hash string"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_separation_produces_distinct_hashes() {
        let payload = b"common test payload for block and tx";

        let block_h = BlockHash::hash(payload);
        let tx_h = TransactionHash::hash(payload);
        let tx_id = TransactionID::hash(payload);
        let branch_h = MerkleBranchHash::hash(payload);
        let ecdsa_h = TransactionSigningHashECDSA::hash(payload);

        assert_ne!(block_h, tx_h);
        assert_ne!(block_h, tx_id);
        assert_ne!(tx_h, tx_id);
        assert_ne!(block_h, branch_h);
        assert_ne!(block_h, ecdsa_h);
        assert_ne!(block_h, Hash::ZERO);
        assert_ne!(ecdsa_h, Hash::ZERO);
    }

    #[test]
    fn test_streaming_update_matches_one_shot_hash() {
        let data = b"streaming incremental data test";

        let one_shot = BlockHash::hash(data);

        let mut hasher = BlockHash::new();
        hasher.update(&data[..10]);
        hasher.update(&data[10..]);
        let incremental = hasher.finalize();

        assert_eq!(one_shot, incremental);
    }

    #[test]
    fn test_hash_hex_serde_roundtrip() {
        let original = BlockHash::hash(b"serialization target");
        let json = serde_json::to_string(&original).unwrap();

        let deserialized: Hash = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);

        let with_prefix = format!("\"0x{}\"", original);
        let de_prefix: Hash = serde_json::from_str(&with_prefix).unwrap();
        assert_eq!(original, de_prefix);
    }

    #[test]
    fn test_le_u64_roundtrip() {
        let words = [123456789u64, 987654321u64, 42u64, 0xdeadbeefu64];
        let hash = Hash::from_le_u64(words);
        assert_eq!(hash.to_le_u64(), words);
    }
}
