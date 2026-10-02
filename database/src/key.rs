//! Database key representations and prefix builders.

use smallvec::SmallVec;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DbKey {
    bytes: SmallVec<[u8; 36]>,
}

impl DbKey {
    pub fn new(prefix: &[u8], key: &[u8]) -> Self {
        let mut bytes = SmallVec::with_capacity(prefix.len() + key.len());
        bytes.extend_from_slice(prefix);
        bytes.extend_from_slice(key);
        Self { bytes }
    }

    pub fn from_slice(bytes: &[u8]) -> Self {
        Self {
            bytes: SmallVec::from_slice(bytes),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl AsRef<[u8]> for DbKey {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
