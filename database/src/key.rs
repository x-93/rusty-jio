use std::{
    fmt::{Debug, Display},
    str,
};

pub const SEP: u8 = b'/';
pub const SEP_SIZE: usize = 1;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbKey {
    path: Vec<u8>,
    prefix_len: usize,
}

impl DbKey {
    pub fn new<TKey>(prefix: &[u8], key: TKey) -> Self
    where
        TKey: Clone + AsRef<[u8]>,
    {
        Self {
            path: prefix.iter().chain(std::iter::once(&SEP)).chain(key.as_ref().iter()).copied().collect(),
            prefix_len: prefix.len() + SEP_SIZE, // Include `SEP` as part of the prefix
        }
    }

    pub fn prefix_only(prefix: &[u8]) -> Self {
        Self::new(prefix, [])
    }

    /// Add a bucket to the DbKey; this expands the prefix length.
    pub fn add_bucket<TBucket>(&mut self, bucket: TBucket)
    where
        TBucket: Copy + AsRef<[u8]>,
    {
        self.path.extend(bucket.as_ref().iter());
        self.prefix_len += bucket.as_ref().len();
    }

    pub fn prefix_len(&self) -> usize {
        self.prefix_len
    }
}

impl AsRef<[u8]> for DbKey {
    fn as_ref(&self) -> &[u8] {
        &self.path
    }
}

impl Display for DbKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (prefix, key) = self.path.split_at(self.prefix_len);
        // We expect the prefix to be human readable
        if let Ok(s) = str::from_utf8(prefix) {
            f.write_str(s)?;
        } else {
            // Otherwise fallback to hex encoding
            let mut hex = vec![0u8; (prefix.len() - SEP_SIZE) * 2];
            faster_hex::hex_encode(&prefix[..prefix.len() - SEP_SIZE], &mut hex).map_err(|_| std::fmt::Error)?;
            let hex_str = str::from_utf8(&hex).map_err(|_| std::fmt::Error)?;
            f.write_str(hex_str)?;
            f.write_str("/")?;
        }
        // Key is formatted as hex
        let mut key_hex = vec![0u8; key.len() * 2];
        faster_hex::hex_encode(key, &mut key_hex).map_err(|_| std::fmt::Error)?;
        let key_str = str::from_utf8(&key_hex).map_err(|_| std::fmt::Error)?;
        f.write_str(key_str)
    }
}

impl Debug for DbKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jio_hashes::Hash;

    #[test]
    fn test_key_display() {
        let key1 = DbKey::new(b"human-readable", Hash::from_bytes([0xaa; 32]));
        let key2 = DbKey::new(&[0xC0, 0xC1, 0xF5, 0xF6], Hash::from_bytes([0xbb; 32]));
        let key3 = DbKey::prefix_only(b"direct-prefix");

        assert!(key1.to_string().starts_with("human-readable/"));
        assert!(key2.to_string().starts_with("c0c1f5f6/"));
        assert_eq!(key3.to_string(), "direct-prefix/");
    }
}
