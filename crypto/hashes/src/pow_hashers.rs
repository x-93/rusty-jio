//! Proof-of-Work hash algorithms for the Jio network.
//!
//! Includes the memory-dependent JioHashV1 mining algorithm and header hashing helpers.

use crate::Hash;
use blake3::Hasher as Blake3Hasher;

// -----------------------------------------------------------------------------
// JioHashV1 reference memory-mixing mining algorithm implementation
// -----------------------------------------------------------------------------

const DOMAIN: &[u8] = b"JioHashV1";
pub const JIOHASH_V1_SCRATCHPAD_SIZE: usize = 1024 * 1024;
pub const JIOHASH_V1_BLOCK_SIZE: usize = 64;
pub const JIOHASH_V1_BLOCK_COUNT: usize = JIOHASH_V1_SCRATCHPAD_SIZE / JIOHASH_V1_BLOCK_SIZE;
pub const JIOHASH_V1_ITERATIONS: u32 = 64;
pub const JIOHASH_V1_OUTPUT_SIZE: usize = 32;

const STAGE_SEED: u8 = 0x01;
const STAGE_MIX: u8 = 0x02;
const STAGE_MUTATE: u8 = 0x03;
const STAGE_FINAL: u8 = 0x04;
const STAGE_INIT: u8 = 0x10;

#[derive(Clone)]
pub struct JioHashV1 {
    pow_hash: Hash,
    seed: [u8; 32],
    nonce: u64,
    scratchpad: Vec<u8>,
}

impl std::fmt::Debug for JioHashV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JioHashV1")
            .field("pow_hash", &self.pow_hash)
            .field("seed", &self.seed)
            .field("nonce", &self.nonce)
            .field("scratchpad_len", &self.scratchpad.len())
            .finish()
    }
}

impl JioHashV1 {
    pub fn new(pow_hash: Hash, nonce: u64) -> Self {
        let seed = derive_seed(pow_hash, nonce);
        let scratchpad = initialize_scratchpad(&seed);
        Self {
            pow_hash,
            seed,
            nonce,
            scratchpad,
        }
    }

    pub fn finalize(mut self) -> Hash {
        let final_state = self.mix();
        finalize_hash(self.pow_hash, self.seed, final_state, self.nonce)
    }

    fn mix(&mut self) -> [u8; 32] {
        let mut state = self.seed;
        for iteration in 0..JIOHASH_V1_ITERATIONS {
            let index = calculate_scratchpad_index(&state);
            let offset = index * JIOHASH_V1_BLOCK_SIZE;
            let mut memory_block = [0u8; JIOHASH_V1_BLOCK_SIZE];
            memory_block.copy_from_slice(&self.scratchpad[offset..offset + JIOHASH_V1_BLOCK_SIZE]);
            state = mix_state(state, memory_block, iteration);
            mutate_scratchpad(&mut self.scratchpad, index, &state);
        }
        state
    }

    pub fn seed(&self) -> [u8; 32] {
        self.seed
    }

    pub fn pow_hash(&self) -> Hash {
        self.pow_hash
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }
}

pub fn hash_header(header: &[u8]) -> Hash {
    let mut hasher = Blake3Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(header);
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

pub fn hash(pow_hash: Hash, nonce: u64) -> Hash {
    JioHashV1::new(pow_hash, nonce).finalize()
}

fn derive_seed(pow_hash: Hash, nonce: u64) -> [u8; 32] {
    let mut hasher = Blake3Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(&[STAGE_SEED]);
    hasher.update(pow_hash.as_ref());
    hasher.update(&nonce.to_le_bytes());
    *hasher.finalize().as_bytes()
}

fn initialize_scratchpad(seed: &[u8; 32]) -> Vec<u8> {
    let mut scratchpad = vec![0u8; JIOHASH_V1_SCRATCHPAD_SIZE];
    for index in 0..JIOHASH_V1_BLOCK_COUNT {
        let block = initialize_scratchpad_block(seed, index as u32);
        let offset = index * JIOHASH_V1_BLOCK_SIZE;
        scratchpad[offset..offset + JIOHASH_V1_BLOCK_SIZE].copy_from_slice(&block);
    }
    scratchpad
}

fn initialize_scratchpad_block(seed: &[u8; 32], index: u32) -> [u8; JIOHASH_V1_BLOCK_SIZE] {
    let mut hasher = Blake3Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(&[STAGE_INIT]);
    hasher.update(seed);
    hasher.update(&index.to_le_bytes());
    let mut output = [0u8; JIOHASH_V1_BLOCK_SIZE];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut output);
    output
}

#[inline(always)]
fn calculate_scratchpad_index(state: &[u8; 32]) -> usize {
    let value = u32::from_le_bytes([state[0], state[1], state[2], state[3]]);
    (value as usize) % JIOHASH_V1_BLOCK_COUNT
}

fn mix_state(state: [u8; 32], memory_block: [u8; JIOHASH_V1_BLOCK_SIZE], iteration: u32) -> [u8; 32] {
    let mut hasher = Blake3Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(&[STAGE_MIX]);
    hasher.update(&state);
    hasher.update(&memory_block);
    hasher.update(&iteration.to_le_bytes());
    *hasher.finalize().as_bytes()
}

fn mutate_scratchpad(scratchpad: &mut [u8], index: usize, state: &[u8; 32]) {
    let mut hasher = Blake3Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(&[STAGE_MUTATE]);
    hasher.update(state);
    let mut mutation = [0u8; JIOHASH_V1_BLOCK_SIZE];
    let mut reader = hasher.finalize_xof();
    reader.fill(&mut mutation);
    let offset = index * JIOHASH_V1_BLOCK_SIZE;
    let block = &mut scratchpad[offset..offset + JIOHASH_V1_BLOCK_SIZE];
    for (byte, mask) in block.iter_mut().zip(mutation) {
        *byte ^= mask;
    }
}

fn finalize_hash(pow_hash: Hash, seed: [u8; 32], final_state: [u8; 32], nonce: u64) -> Hash {
    let mut hasher = Blake3Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(&[STAGE_FINAL]);
    hasher.update(pow_hash.as_ref());
    hasher.update(&seed);
    hasher.update(&final_state);
    hasher.update(&nonce.to_le_bytes());
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jiohash_v1_determinism() {
        let pow_hash = Hash::from_u64_word(123456);
        let nonce = 42;
        let h1 = hash(pow_hash, nonce);
        let h2 = hash(pow_hash, nonce);
        assert_eq!(h1, h2);
        assert_ne!(h1, Hash::default());
    }

    #[test]
    fn test_jiohash_v1_different_nonces() {
        let pow_hash = Hash::from_u64_word(123456);
        let h1 = hash(pow_hash, 1);
        let h2 = hash(pow_hash, 2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_header() {
        let header = b"test header bytes for jio pow";
        let h = hash_header(header);
        assert_ne!(h, Hash::default());
    }
}
