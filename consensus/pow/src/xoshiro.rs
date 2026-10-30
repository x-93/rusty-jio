/// The XoShiRo256++ pseudo-random number generator.
/// State consists of 4 64-bit unsigned integers.
#[derive(Clone, Debug)]
pub struct XoShiRo256PlusPlus {
    s: [u64; 4],
}

impl XoShiRo256PlusPlus {
    /// Initializes the PRNG deterministically from a 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let mut s = [0u64; 4];
        for (i, item) in s.iter_mut().enumerate() {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&seed[i * 8..(i + 1) * 8]);
            *item = u64::from_le_bytes(buf);
        }
        // Ensure state is not all zeros
        if s[0] == 0 && s[1] == 0 && s[2] == 0 && s[3] == 0 {
            s[0] = 0x9E3779B97F4A7C15;
            s[1] = 0xBF58476D1CE4E5B9;
            s[2] = 0x94D049BB133111EB;
            s[3] = 0x46D32A15E7C81274;
        }
        Self { s }
    }

    /// Generates the next 64-bit pseudo-random unsigned integer.
    pub fn next_u64(&mut self) -> u64 {
        let result = (self.s[0].wrapping_add(self.s[3]))
            .rotate_left(23)
            .wrapping_add(self.s[0]);

        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        result
    }
}
