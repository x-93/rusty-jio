use crate::xoshiro::XoShiRo256PlusPlus;

/// A 64x64 matrix of 4-bit nibbles (values 0..=15).
#[derive(Clone, Debug)]
pub struct Matrix64x64 {
    pub rows: [[u8; 64]; 64],
}

impl Matrix64x64 {
    /// Generates a deterministic 64x64 nibble matrix using XoShiRo256++.
    pub fn generate(prng: &mut XoShiRo256PlusPlus) -> Self {
        let mut rows = [[0u8; 64]; 64];
        for row in &mut rows {
            for chunk in 0..4 {
                let val = prng.next_u64();
                for nibble_idx in 0..16 {
                    let col = chunk * 16 + nibble_idx;
                    row[col] = ((val >> (nibble_idx * 4)) & 0x0F) as u8;
                }
            }
        }
        Self { rows }
    }

    /// Multiplies the matrix by a 64-element 4-bit vector, shifts by 10, and XORs:
    /// out_nibble[i] = ((sum_{j=0}^{63} M[i][j] * v[j]) >> 10) ^ v[i]
    pub fn multiply(&self, vector: &[u8; 64]) -> [u8; 64] {
        let mut out = [0u8; 64];
        for i in 0..64 {
            let mut sum: u32 = 0;
            for j in 0..64 {
                sum = sum.wrapping_add((self.rows[i][j] as u32) * (vector[j] as u32));
            }
            let shifted = (sum >> 10) as u8;
            out[i] = (shifted ^ vector[i]) & 0x0F;
        }
        out
    }
}
