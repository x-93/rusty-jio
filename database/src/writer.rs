use rocksdb::WriteBatch;

use crate::prelude::DB;

/// Trait defining common write operations for immediate and batched database mutations.
pub trait DbWriter {
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), rocksdb::Error>;
    fn delete(&mut self, key: &[u8]) -> Result<(), rocksdb::Error>;
}

/// A writer that applies changes directly and immediately to the database.
pub struct DirectDbWriter<'a> {
    db: &'a DB,
}

impl<'a> DirectDbWriter<'a> {
    pub fn new(db: &'a DB) -> Self {
        Self { db }
    }
}

impl DbWriter for DirectDbWriter<'_> {
    #[inline(always)]
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), rocksdb::Error> {
        self.db.put(key, value)
    }

    #[inline(always)]
    fn delete(&mut self, key: &[u8]) -> Result<(), rocksdb::Error> {
        self.db.delete(key)
    }
}

/// A writer that collects changes into an atomic `WriteBatch` to be committed all at once.
pub struct BatchDbWriter<'a> {
    db: &'a DB,
    batch: WriteBatch,
}

impl<'a> BatchDbWriter<'a> {
    pub fn new(db: &'a DB) -> Self {
        Self {
            db,
            batch: WriteBatch::default(),
        }
    }

    /// Commits all batched operations atomically to the underlying database.
    pub fn commit(self) -> Result<(), rocksdb::Error> {
        self.db.write(self.batch)
    }
}

impl DbWriter for BatchDbWriter<'_> {
    #[inline(always)]
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), rocksdb::Error> {
        self.batch.put(key, value);
        Ok(())
    }

    #[inline(always)]
    fn delete(&mut self, key: &[u8]) -> Result<(), rocksdb::Error> {
        self.batch.delete(key);
        Ok(())
    }
}
