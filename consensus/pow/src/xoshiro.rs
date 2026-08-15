use jio_hashes::Hash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Xoshiro256PlusPlus {
    s: [u64; 4],
}

impl Xoshiro256PlusPlus {
    pub fn new(seed: [u64; 4]) -> Self {
        let mut s = seed;
        if s == [0, 0, 0, 0] {
            s = [1, 2, 3, 4];
        }
        Self { s }
    }

    pub fn from_hash(hash: &Hash) -> Self {
        let bytes = hash.as_bytes();
        let s0 = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let s1 = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let s2 = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let s3 = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
        Self::new([s0, s1, s2, s3])
    }

    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[0]
            .wrapping_add(self.s[3])
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
