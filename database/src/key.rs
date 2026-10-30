use crate::registry::DatabaseStorePrefixes;

/// Represents a composite key stored in RocksDB.
/// Formed by prefix bytes (representing the store / table) followed by item key bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DbKey {
    bytes: Vec<u8>,
}

impl DbKey {
    pub fn new(prefix: &[u8], key: &[u8]) -> Self {
        let mut bytes = Vec::with_capacity(prefix.len() + key.len());
        bytes.extend_from_slice(prefix);
        bytes.extend_from_slice(key);
        Self { bytes }
    }

    pub fn from_store(store: DatabaseStorePrefixes, key: &[u8]) -> Self {
        Self::new(store.as_ref(), key)
    }

    pub fn prefix_only(prefix: &[u8]) -> Self {
        Self {
            bytes: prefix.to_vec(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl AsRef<[u8]> for DbKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
