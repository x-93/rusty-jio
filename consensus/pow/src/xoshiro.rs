// xoshiro.rs
//
// Xoshiro256++ (Xor-Shift-Rotate 256-bit with ++ scrambler)
// Designed by David Blackman and Sebastiano Vigna (2019).
//
// Role in rusty-jio:
// Deterministically generates the pseudo-random 64x64 u16 HeavyHash matrix
// from the 32-byte `pre_pow_hash` (4 x u64).
//
// Properties:
// - Period: 2^256 - 1
// - State: exactly four 64-bit words ([u64; 4])
// - Scrambler: ++ (rot_left(s0 + s3, 23) + s0)
// - Exceptional speed: ~0.5 to 1.0 CPU cycles per 64-bit word.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Xoshiro256PlusPlus(pub [u64; 4]);

impl Xoshiro256PlusPlus {
    /// Initialize with a 256-bit seed (four u64 words).
    ///
    /// If the seed is all zeros (which is an invalid state for Xoshiro),
    /// we fallback to a non-zero default seed (the Golden Ratio constant).
    #[inline(always)]
    pub fn new(seed: [u64; 4]) -> Self {
        if seed == [0, 0, 0, 0] {
            // Fallback non-zero seed (SplitMix64 Golden Ratio constants)
            Self([
                0x9e3779b97f4a7c15,
                0xbf58476d1ce4e5b9,
                0x94d049bb133111eb,
                0x4bfda9c4c64a3239,
            ])
        } else {
            Self(seed)
        }
    }

    /// Generate the next pseudo-random 64-bit integer and advance internal state.
    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        let result = (self.0[0].wrapping_add(self.0[3]))
            .rotate_left(23)
            .wrapping_add(self.0[0]);

        let t = self.0[1] << 17;

        self.0[2] ^= self.0[0];
        self.0[3] ^= self.0[1];
        self.0[1] ^= self.0[2];
        self.0[0] ^= self.0[3];

        self.0[2] ^= t;
        self.0[3] = self.0[3].rotate_left(45);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism() {
        let seed = [1, 2, 3, 4];
        let mut rng1 = Xoshiro256PlusPlus::new(seed);
        let mut rng2 = Xoshiro256PlusPlus::new(seed);

        for _ in 0..1000 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_zero_seed_protection() {
        let mut rng = Xoshiro256PlusPlus::new([0, 0, 0, 0]);
        // Must not get stuck in a zero-loop
        let val = rng.next_u64();
        assert_ne!(val, 0);
    }
}
