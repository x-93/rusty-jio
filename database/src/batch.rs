use crate::key::DbKey;
use rocksdb::WriteBatch;

/// An atomic write batch wrapper for RocksDB operations.
#[derive(Default)]
pub struct BatchDbWriter {
    batch: WriteBatch,
    count: usize,
}

impl BatchDbWriter {
    pub fn new() -> Self {
        Self {
            batch: WriteBatch::default(),
            count: 0,
        }
    }

    /// Add a raw key-value put to the batch.
    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.batch.put(key, value);
        self.count += 1;
    }

    /// Add a raw key deletion to the batch.
    pub fn delete(&mut self, key: &[u8]) {
        self.batch.delete(key);
        self.count += 1;
    }

    /// Add a prefixed key-value put to the batch.
    pub fn put_prefixed(&mut self, prefix: &[u8], key: &[u8], value: &[u8]) {
        let db_key = DbKey::new(prefix, key);
        self.put(db_key.as_bytes(), value);
    }

    /// Add a prefixed key deletion to the batch.
    pub fn delete_prefixed(&mut self, prefix: &[u8], key: &[u8]) {
        let db_key = DbKey::new(prefix, key);
        self.delete(db_key.as_bytes());
    }

    /// Number of operations currently accumulated in this batch.
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn clear(&mut self) {
        self.batch.clear();
        self.count = 0;
    }

    pub fn into_inner(self) -> WriteBatch {
        self.batch
    }

    pub fn as_inner(&self) -> &WriteBatch {
        &self.batch
    }

    pub fn as_inner_mut(&mut self) -> &mut WriteBatch {
        &mut self.batch
    }
}

