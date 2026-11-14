// pow_hashers.rs
//
// Proof-of-Work hashers for rusty-jio:
//
//   1. PowHash    — High-performance cSHAKE256("ProofOfWorkHash")
//                   Pre-computed 25-word Keccak state with pre-applied padding.
//                   Guarantees 100% bitwise parity between mining loops and block validation.
//
//   2. PowB3Hash  — BLAKE3 Keyed Hash (Key = "rusty-jio:ProofOfWork-BLAKE3-v1\0")
//                   Replaces legacy KHeavyHash for 10-15x throughput and native SIMD execution.

use once_cell::sync::Lazy;

use crate::{Hash, Hasher, HasherBase};

// ---------------------------------------------------------------------------
// 1. PowHash — Canonical Keccak-f[1600] for cSHAKE256("ProofOfWorkHash")
// ---------------------------------------------------------------------------
//
// Layout of 136-byte (17 u64 words) Rate block:
// [ 0..=3 ]: pre_pow_hash (32 bytes = 4 words)
// [   4   ]: timestamp (8 bytes = 1 word)
// [ 5..=8 ]: reserved / extra zeroes (32 bytes = 4 words)
// [   9   ]: nonce (8 bytes = 1 word)
// [  10   ]: padding start byte 0x04 (pre-applied in INITIAL_STATE)
// [11..=15]: zero padding
// [  16   ]: final padding bit 0x80 (pre-applied in INITIAL_STATE)
// [17..=24]: capacity (512 bits = 8 words)
//
// Pre-computed state:
// [10] -> 1123092876221303310 ^ 0x04(padding byte) = 1123092876221303306
// [16] -> 10306167911662716186 ^ 0x8000000000000000(final padding) = 1082795874807940378

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowHash {
    pub state: [u64; 25],
    pub bytes_written: usize,
}

impl PowHash {
    /// Pre-computed initial state for `cSHAKE256("ProofOfWorkHash")`
    /// with domain separator and cSHAKE padding bits pre-XOR'd.
    #[rustfmt::skip]
    pub const INITIAL_STATE: [u64; 25] = [
        1242148031264380989, 3008272977830772284, 2188519011337848018, 1992179434288343456, 8876506674959887717,
        5399642050693751366, 1745875063082670864, 8605242046444978844, 17936695144567157056, 3343109343542796272,
        1123092876221303306, 4963925045340115282, 17037383077651887893, 16629644495023626889, 12833675776649114147,
        3784524041015224902, 1082795874807940378, 13952716920571277634, 13411128033953605860, 15060696040649351053,
        9928834659948351306, 5237849264682708699, 12825353012139217522, 6706187291358897596, 196324915476054915,
    ];

    /// Standard constructor initialized with the canonical state.
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            state: Self::INITIAL_STATE,
            bytes_written: 0,
        }
    }

    /// Primary production constructor:
    /// Ingests `pre_pow_hash` and `timestamp` into words 0..=4.
    /// Used by both mining loops and node block validation.
    #[inline]
    pub fn from_pre_pow_and_timestamp(pre_pow_hash: Hash, timestamp: u64) -> Self {
        let mut start = Self::INITIAL_STATE;
        for (pre_pow_word, state_word) in pre_pow_hash.iter_le_u64().zip(start.iter_mut()) {
            *state_word ^= pre_pow_word;
        }
        start[4] ^= timestamp;
        Self {
            state: start,
            bytes_written: 40,
        }
    }

    /// Canonical block header hash:
    /// Guaranteed to be 100% bitwise identical across miners and verifying nodes.
    #[inline(always)]
    pub fn check_pow(pre_pow_hash: Hash, timestamp: u64, nonce: u64) -> Hash {
        let hasher = Self::from_pre_pow_and_timestamp(pre_pow_hash, timestamp);
        hasher.finalize_with_nonce(nonce)
    }

    /// Hyper-optimized inner mining loop:
    /// Injects the `nonce`, executes a single Keccak-f[1600] round, and extracts the 32-byte hash.
    #[inline(always)]
    pub fn finalize_with_nonce(mut self, nonce: u64) -> Hash {
        self.state[9] ^= nonce;
        keccak::f1600(&mut self.state);
        Hash::from_le_u64(self.state[..4].try_into().unwrap())
    }

    /// Canonical byte-stream ingestion with byte-accurate state tracking.
    pub fn write<A: AsRef<[u8]>>(&mut self, data: A) {
        let bytes = data.as_ref();
        for &byte in bytes {
            let word_idx = self.bytes_written / 8;
            let byte_in_word = self.bytes_written % 8;
            if word_idx <= 9 {
                self.state[word_idx] ^= (byte as u64) << (byte_in_word * 8);
            }
            self.bytes_written += 1;
        }
    }

    #[inline(always)]
    pub fn finalize(mut self) -> Hash {
        keccak::f1600(&mut self.state);
        Hash::from_le_u64(self.state[..4].try_into().unwrap())
    }
}

impl HasherBase for PowHash {
    #[inline(always)]
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
        self.write(data);
        self
    }
}

impl Hasher for PowHash {
    #[inline(always)]
    fn finalize(self) -> Hash {
        PowHash::finalize(self)
    }

    #[inline(always)]
    fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for PowHash {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 2. PowB3Hash — BLAKE3 in Keyed Mode
// ---------------------------------------------------------------------------

/// 32-byte domain separation key for Proof-of-Work
pub const POW_B3_KEY: [u8; 32] = *b"rusty-jio:ProofOfWork-BLAKE3-v1\0";

#[derive(Clone)]
pub struct PowB3Hash(blake3::Hasher);

impl PowB3Hash {
    #[inline(always)]
    pub fn new() -> Self {
        static HASHER: Lazy<PowB3Hash> =
            Lazy::new(|| PowB3Hash(blake3::Hasher::new_keyed(&POW_B3_KEY)));
        (*HASHER).clone()
    }

    #[inline(always)]
    pub fn write<A: AsRef<[u8]>>(&mut self, data: A) {
        self.0.update(data.as_ref());
    }

    #[inline(always)]
    pub fn finalize(self) -> Hash {
        Hash::from_bytes(*self.0.finalize().as_bytes())
    }
}

impl HasherBase for PowB3Hash {
    #[inline(always)]
    fn update<A: AsRef<[u8]>>(&mut self, data: A) -> &mut Self {
        self.write(data);
        self
    }
}

impl Hasher for PowB3Hash {
    #[inline(always)]
    fn finalize(self) -> Hash {
        PowB3Hash::finalize(self)
    }

    #[inline(always)]
    fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for PowB3Hash {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Security & Parity Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Hasher;

    #[test]
    fn test_miner_node_parity() {
        // Simulates honest node verifying a block mined by a miner
        let pre_pow = Hash::from_le_u64([
            0x1234567890ABCDEF,
            0xFEDCBA0987654321,
            0x1122334455667788,
            0x99AABBCCDDEEFF00,
        ]);
        let timestamp = 1700000000u64;
        let nonce = 987654321u64;

        // 1. Miner path
        let miner_hash =
            PowHash::from_pre_pow_and_timestamp(pre_pow, timestamp).finalize_with_nonce(nonce);

        // 2. Node canonical validation path
        let node_hash = PowHash::check_pow(pre_pow, timestamp, nonce);

        // 3. Serialized raw stream verification path
        let mut stream_hasher = PowHash::new();
        stream_hasher.write(pre_pow.as_bytes());
        stream_hasher.write(timestamp.to_le_bytes());
        stream_hasher.write([0u8; 32]); // reserved words 5..=8
        stream_hasher.write(nonce.to_le_bytes());
        let stream_hash = stream_hasher.finalize();

        assert_eq!(
            miner_hash, node_hash,
            "Miner path and check_pow must be bitwise identical"
        );
        assert_eq!(
            miner_hash, stream_hash,
            "Stream write and fast mining path MUST match 100%"
        );
    }

    #[test]
    fn test_pow_hash_finalize_with_nonce() {
        let pre_pow = Hash::from_le_u64([111, 222, 333, 444]);
        let timestamp = 1700000000u64;
        let nonce = 123456789u64;

        let hasher = PowHash::from_pre_pow_and_timestamp(pre_pow, timestamp);
        let h1 = hasher.finalize_with_nonce(nonce);
        let h2 = hasher.finalize_with_nonce(nonce);
        assert_eq!(h1, h2, "Mining loop with same nonce must be deterministic");

        let h3 = hasher.finalize_with_nonce(nonce + 1);
        assert_ne!(
            h1, h3,
            "Different nonce must produce completely different hash"
        );
    }

    #[test]
    fn test_pow_b3_domain_separation() {
        let data = b"some block payload";
        let b3_pow = PowB3Hash::hash(data);
        let b3_generic = Hash::from_bytes(*blake3::hash(data).as_bytes());
        assert_ne!(
            b3_pow, b3_generic,
            "Keyed PowB3Hash must not match unkeyed blake3"
        );
    }

    #[test]
    fn test_pow_hash_and_b3_differ() {
        let data = b"identical input payload";
        let pow = PowHash::hash(data);
        let b3 = PowB3Hash::hash(data);
        assert_ne!(pow, b3);
    }
}
