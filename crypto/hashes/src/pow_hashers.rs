//! Matrix-based PoW hashers for the Jio mining algorithm.
//!
//! Includes Matrix multiplication over 4-bit nibbles, PowHash, PowB3Hash,
//! and the memory-dependent JioHashV1 mining algorithm.

use crate::Hash;
use blake3::Hasher as Blake3Hasher;

pub const MATRIX_SIZE: usize = 64;

/// 64x64 matrix of 4-bit elements used in the PoW heavy-hash algorithm.
#[derive(Clone, Debug)]
pub struct Matrix {
    pub rows: [[u8; MATRIX_SIZE]; MATRIX_SIZE],
}

impl Matrix {
    /// Generate a 64x64 matrix deterministically from a 32-byte hash.
    pub fn generate(seed: Hash) -> Self {
        let mut hasher = Blake3Hasher::new();
        hasher.update(b"JioPoWMatrix");
        hasher.update(seed.as_ref());
        let mut reader = hasher.finalize_xof();

        let mut rows = [[0u8; MATRIX_SIZE]; MATRIX_SIZE];
        let mut buf = [0u8; MATRIX_SIZE * MATRIX_SIZE / 2];
        reader.fill(&mut buf);

        for i in 0..MATRIX_SIZE {
            for j in 0..MATRIX_SIZE {
                let byte_idx = (i * MATRIX_SIZE + j) / 2;
                let nibble = if (i * MATRIX_SIZE + j) % 2 == 0 {
                    buf[byte_idx] & 0x0F
                } else {
                    (buf[byte_idx] >> 4) & 0x0F
                };
                rows[i][j] = nibble;
            }
        }
        Self { rows }
    }

    /// Multiply matrix by a 64-nibble vector (from 32-byte hash).
    /// Computes $v'_i = \sum_{j=0}^{63} M_{i,j} \cdot v_j \pmod{16}$.
    pub fn multiply_vector(&self, vector: &[u8; 32]) -> [u8; 32] {
        let mut vec_nibbles = [0u8; MATRIX_SIZE];
        for i in 0..32 {
            vec_nibbles[i * 2] = vector[i] & 0x0F;
            vec_nibbles[i * 2 + 1] = (vector[i] >> 4) & 0x0F;
        }

        let mut res_nibbles = [0u8; MATRIX_SIZE];
        for i in 0..MATRIX_SIZE {
            let mut sum = 0u32;
            for j in 0..MATRIX_SIZE {
                sum += (self.rows[i][j] as u32) * (vec_nibbles[j] as u32);
            }
            res_nibbles[i] = (sum & 0x0F) as u8;
        }

        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = res_nibbles[i * 2] | (res_nibbles[i * 2 + 1] << 4);
        }
        out
    }
}

/// Matrix-based PoW hasher for the mining algorithm.
#[derive(Clone, Debug)]
pub struct PowHash {
    matrix: Matrix,
    pre_pow_hash: Hash,
}

impl PowHash {
    pub fn new(pre_pow_hash: Hash) -> Self {
        let matrix = Matrix::generate(pre_pow_hash);
        Self { matrix, pre_pow_hash }
    }

    /// Calculate the matrix PoW hash for a given nonce.
    pub fn calculate_pow(&self, nonce: u64) -> Hash {
        // Stage 1: hash pre_pow_hash + nonce
        let mut h1 = Blake3Hasher::new();
        h1.update(b"JioPoWStage1");
        h1.update(self.pre_pow_hash.as_ref());
        h1.update(&nonce.to_le_bytes());
        let v = *h1.finalize().as_bytes();

        // Stage 2: matrix multiplication
        let mv = self.matrix.multiply_vector(&v);

        // Stage 3: final Blake3 digest
        let mut h2 = Blake3Hasher::new();
        h2.update(b"JioPoWStage2");
        h2.update(&mv);
        h2.update(self.pre_pow_hash.as_ref());
        h2.update(&nonce.to_le_bytes());
        Hash::from_bytes(*h2.finalize().as_bytes())
    }
}

/// Blake3-optimized matrix PoW hasher.
#[derive(Clone, Debug)]
pub struct PowB3Hash {
    inner: PowHash,
}

impl PowB3Hash {
    pub fn new(pre_pow_hash: Hash) -> Self {
        Self {
            inner: PowHash::new(pre_pow_hash),
        }
    }

    #[inline(always)]
    pub fn calculate_pow(&self, nonce: u64) -> Hash {
        self.inner.calculate_pow(nonce)
    }
}

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
    fn test_matrix_generation_and_multiply() {
        let seed = Hash::from_u64_word(123456);
        let matrix = Matrix::generate(seed);
        let vec = [0x55u8; 32];
        let result = matrix.multiply_vector(&vec);
        assert_ne!(result, [0u8; 32]);
    }

    #[test]
    fn test_pow_hash() {
        let pre_pow = Hash::from_u64_word(9999);
        let pow_hasher = PowHash::new(pre_pow);
        let h1 = pow_hasher.calculate_pow(1);
        let h2 = pow_hasher.calculate_pow(2);
        assert_ne!(h1, h2);
    }
}