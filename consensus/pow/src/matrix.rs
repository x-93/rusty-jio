use blake3::Hasher as Blake3Hasher;
use jio_hashes::Hash;

pub const MATRIX_SIZE: usize = 64;

/// 64x64 matrix of 4-bit elements used in the PoW heavy-hash algorithm.
#[derive(Clone, Debug, PartialEq, Eq)]
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

        for (i, row) in rows.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let idx = i * MATRIX_SIZE + j;
                let byte_idx = idx / 2;
                let nibble = if idx.is_multiple_of(2) {
                    buf[byte_idx] & 0x0F
                } else {
                    (buf[byte_idx] >> 4) & 0x0F
                };
                *cell = nibble;
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
        for (i, res) in res_nibbles.iter_mut().enumerate() {
            let mut sum = 0u32;
            for (j, &vec_val) in vec_nibbles.iter().enumerate() {
                sum += (self.rows[i][j] as u32) * (vec_val as u32);
            }
            *res = (sum & 0x0F) as u8;
        }

        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = res_nibbles[i * 2] | (res_nibbles[i * 2 + 1] << 4);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_generate_and_multiply() {
        let seed = Hash::from_u64_word(42);
        let matrix = Matrix::generate(seed);
        let vector = [0x55u8; 32];
        let result = matrix.multiply_vector(&vector);
        assert_ne!(result, [0u8; 32]);
    }
}