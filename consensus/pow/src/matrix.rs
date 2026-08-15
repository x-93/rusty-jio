use crate::xoshiro::Xoshiro256PlusPlus;
use jio_hashes::Hash;

pub const MATRIX_SIZE: usize = 64;

#[derive(Clone, Debug)]
pub struct Matrix {
    pub rows: [[u16; MATRIX_SIZE]; MATRIX_SIZE],
}

impl Matrix {
    pub fn generate(pre_pow_hash: &Hash) -> Self {
        let mut prng = Xoshiro256PlusPlus::from_hash(pre_pow_hash);
        let mut rows = [[0u16; MATRIX_SIZE]; MATRIX_SIZE];

        for i in 0..MATRIX_SIZE {
            for j in (0..MATRIX_SIZE).step_by(16) {
                let val = prng.next_u64();
                for k in 0..16 {
                    rows[i][j + k] = ((val >> (k * 4)) & 0x0F) as u16;
                }
            }
        }

        Self { rows }
    }

    pub fn heavy_hash(&self, input_hash: &Hash) -> Hash {
        let input_bytes = input_hash.as_bytes();
        let mut v = [0u16; MATRIX_SIZE];

        // Unpack 32 bytes into 64 4-bit nibbles
        for i in 0..32 {
            v[i * 2] = (input_bytes[i] & 0x0F) as u16;
            v[i * 2 + 1] = ((input_bytes[i] >> 4) & 0x0F) as u16;
        }

        // Matrix * Vector multiplication
        let mut product = [0u16; MATRIX_SIZE];
        for i in 0..MATRIX_SIZE {
            let mut sum = 0u32;
            for j in 0..MATRIX_SIZE {
                sum += (self.rows[i][j] as u32) * (v[j] as u32);
            }
            // Reduction: 4-bit output
            product[i] = ((sum >> 10) & 0x0F) as u16;
        }

        // Pack 64 4-bit nibbles into 32 bytes
        let mut output_bytes = [0u8; 32];
        for i in 0..32 {
            output_bytes[i] = (product[i * 2] as u8) | ((product[i * 2 + 1] as u8) << 4);
        }

        // Final hashing step
        jio_hashes::blake3(&output_bytes)
    }
}
