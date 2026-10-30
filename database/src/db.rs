use crate::batch::BatchDbWriter;
use crate::error::DatabaseError;
use rocksdb::{DBWithThreadMode, IteratorMode, Options, SingleThreaded};
use std::path::Path;
use std::sync::Arc;

pub type RawDb = DBWithThreadMode<SingleThreaded>;
pub type DbEntry = (Box<[u8]>, Box<[u8]>);
pub type DbEntryResult = Result<DbEntry, DatabaseError>;

/// Thread-safe RocksDB instance wrapper.
pub struct DB {
    inner: RawDb,
}

impl DB {
    /// Open a RocksDB instance at `path` with default optimized options.
    pub fn open_default(path: impl AsRef<Path>) -> Result<Arc<Self>, DatabaseError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_keep_log_file_num(5);
        Self::open(path, opts)
    }

    /// Open a RocksDB instance at `path` with customized options.
    pub fn open(path: impl AsRef<Path>, options: Options) -> Result<Arc<Self>, DatabaseError> {
        let db = RawDb::open(&options, path)?;
        Ok(Arc::new(Self { inner: db }))
    }

    /// Convenience helper for tests: opens a database in a temporary directory.
    pub fn create_temp_db() -> Result<(Arc<Self>, tempfile::TempDir), DatabaseError> {
        let temp_dir = tempfile::tempdir()?;
        let db = Self::open_default(temp_dir.path())?;
        Ok((db, temp_dir))
    }

    /// Read a value by raw key.
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        Ok(self.inner.get(key)?)
    }

    /// Write a key-value pair.
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        self.inner.put(key, value)?;
        Ok(())
    }

    /// Delete a key.
    pub fn delete(&self, key: &[u8]) -> Result<(), DatabaseError> {
        self.inner.delete(key)?;
        Ok(())
    }

    /// Atomically commit a write batch.
    pub fn write_batch(&self, batch: BatchDbWriter) -> Result<(), DatabaseError> {
        self.inner.write(batch.into_inner())?;
        Ok(())
    }

    /// Explicitly flush memtables to disk.
    pub fn flush(&self) -> Result<(), DatabaseError> {
        self.inner.flush()?;
        Ok(())
    }

    /// Iterate over database entries starting from a given mode.
    pub fn iterator(&self, mode: IteratorMode) -> impl Iterator<Item = DbEntryResult> + '_ {
        self.inner
            .iterator(mode)
            .map(|res| res.map_err(DatabaseError::from))
    }

    /// Access the underlying RocksDB instance.
    pub fn raw(&self) -> &RawDb {
        &self.inner
    }
}
