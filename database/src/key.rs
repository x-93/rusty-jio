use jio_utils::hex::ToHex;
use smallvec::SmallVec;
use std::fmt::Debug;

pub type KeyVec = SmallVec<[u8; 36]>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DbKey {
    bytes: KeyVec,
}

impl DbKey {
    pub fn new(bytes: impl AsRef<[u8]>) -> Self {
        Self {
            bytes: KeyVec::from_slice(bytes.as_ref()),
        }
    }

    pub fn prefix(bucket: &[u8], key: &[u8]) -> Self {
        let mut bytes = KeyVec::with_capacity(bucket.len() + key.len());
        bytes.extend_from_slice(bucket);
        bytes.extend_from_slice(key);
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsRef<[u8]> for DbKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Debug for DbKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DbKey({})", self.bytes.to_hex())
    }
}
