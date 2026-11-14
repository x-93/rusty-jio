// matrix.rs
//
// 64x64 Matrix and Vector GEMV (General Matrix-Vector Multiplication)
// Used in the memory-bound Proof-of-Work stage of rusty-jio.
//
// Why 64x64 uint16?
// - Matrix elements: 64 x 64 = 4096 elements.
// - Memory footprint: 4096 x 2 bytes = 8,192 bytes (8 KiB).
// - Fits directly into CPU L1 cache (typically 32-64 KiB), ensuring maximum
//   execution speed on CPUs/GPUs while requiring dedicated memory access on ASICs.
//
// The product vector v' = M x v is computed modulo 2^16 per row.

use crate::xoshiro::Xoshiro256PlusPlus;
use jio_hashes::Hash;

pub const MATRIX_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Matrix(pub [[u16; MATRIX_SIZE]; MATRIX_SIZE]);

impl Matrix {
    /// Deterministically generates a 64x64 matrix of u16 values using Xoshiro256++.
    ///
    /// Every 64-bit output of Xoshiro produces four 16-bit values.
    /// 64 rows * 64 columns = 4096 u16 values = 1024 calls to `next_u64()`.
    pub fn generate(rng: &mut Xoshiro256PlusPlus) -> Self {
        let mut mat = [[0u16; MATRIX_SIZE]; MATRIX_SIZE];
        for row in mat.iter_mut() {
            for chunk in row.as_chunks_mut::<4>().0 {
                let rand_val = rng.next_u64();
                chunk[0] = rand_val as u16;
                chunk[1] = (rand_val >> 16) as u16;
                chunk[2] = (rand_val >> 32) as u16;
                chunk[3] = (rand_val >> 48) as u16;
            }
        }
        Matrix(mat)
    }

    /// Multiply this 64x64 matrix by a 64-element u16 vector.
    ///
    /// product[i] = (sum_{j=0..63} matrix[i][j] * vector[j]) mod 2^16
    #[inline(always)]
    #[allow(clippy::needless_range_loop)]
    pub fn mul_vector(&self, vector: &[u16; MATRIX_SIZE]) -> [u16; MATRIX_SIZE] {
        let mut product = [0u16; MATRIX_SIZE];
        for i in 0..MATRIX_SIZE {
            let mut sum: u32 = 0;
            for j in 0..MATRIX_SIZE {
                sum = sum.wrapping_add((self.0[i][j] as u32) * (vector[j] as u32));
            }
            product[i] = sum as u16;
        }
        product
    }
}

/// Extracts a 64-element u16 vector from a 32-byte `Hash`.
///
/// Each byte contains two 4-bit nibbles:
/// Byte -> (low nibble, high nibble).
/// 32 bytes * 2 nibbles/byte = 64 nibbles (u16 values in 0..=15).
#[inline(always)]
pub fn hash_to_vector(hash: &Hash) -> [u16; MATRIX_SIZE] {
    let mut vec = [0u16; MATRIX_SIZE];
    let bytes = hash.as_bytes();
    for i in 0..32 {
        vec[2 * i] = (bytes[i] & 0x0F) as u16;
        vec[2 * i + 1] = ((bytes[i] >> 4) & 0x0F) as u16;
    }
    vec
}

/// Compresses a 64-element u16 product vector into a 32-byte `Hash`.
///
/// Each u16 product element has its low 4 bits (nibble) extracted.
/// Two nibbles are packed back into one byte:
/// 64 nibbles -> 32 bytes.
#[inline(always)]
pub fn vector_to_hash(vector: &[u16; MATRIX_SIZE]) -> Hash {
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let low = (vector[2 * i] & 0x0F) as u8;
        let high = ((vector[2 * i + 1] & 0x0F) as u8) << 4;
        bytes[i] = low | high;
    }
    Hash::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_generation_determinism() {
        let mut rng1 = Xoshiro256PlusPlus::new([10, 20, 30, 40]);
        let mut rng2 = Xoshiro256PlusPlus::new([10, 20, 30, 40]);

        let mat1 = Matrix::generate(&mut rng1);
        let mat2 = Matrix::generate(&mut rng2);
        assert_eq!(mat1, mat2);
    }

    #[test]
    fn test_vector_extraction_and_compression() {
        let raw_hash = Hash::from_le_u64([0x1234567890abcdef, 0xfedcba0987654321, 100, 200]);
        let vec = hash_to_vector(&raw_hash);
        assert_eq!(vec.len(), 64);

        // Pack back and compare with original (since each nibble was preserved)
        let compressed = vector_to_hash(&vec);
        assert_eq!(raw_hash, compressed);
    }

    #[test]
    fn test_matrix_vector_mul() {
        let mut rng = Xoshiro256PlusPlus::new([1, 2, 3, 4]);
        let mat = Matrix::generate(&mut rng);
        let vec = [1u16; 64];
        let res = mat.mul_vector(&vec);
        assert_eq!(res.len(), 64);
    }
}
