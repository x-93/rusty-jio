//! Database writers for direct and batched atomic persistence.

use super::errors::StoreResult;
use std::sync::Arc;
use parking_lot::Mutex;

pub trait DbWriter {
    fn put(&mut self, key: &[u8], value: &[u8]) -> StoreResult<()>;
    fn delete(&mut self, key: &[u8]) -> StoreResult<()>;
}

impl<T: DbWriter + ?Sized> DbWriter for &mut T {
    fn put(&mut self, key: &[u8], value: &[u8]) -> StoreResult<()> {
        (**self).put(key, value)
    }

    fn delete(&mut self, key: &[u8]) -> StoreResult<()> {
        (**self).delete(key)
    }
}

/// A writer that performs immediate synchronous operations on the database.
pub struct DirectDbWriter<'a> {
    db: &'a super::db::DB,
}

impl<'a> DirectDbWriter<'a> {
    pub fn new(db: &'a super::db::DB) -> Self {
        Self { db }
    }
}

impl<'a> DbWriter for DirectDbWriter<'a> {
    fn put(&mut self, key: &[u8], value: &[u8]) -> StoreResult<()> {
        self.db.put_raw(key, value)
    }

    fn delete(&mut self, key: &[u8]) -> StoreResult<()> {
        self.db.delete_raw(key)
    }
}

pub enum WriteOperation {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

/// A writer that queues write operations and commits them atomically.
#[derive(Default)]
pub struct BatchDbWriter {
    operations: Vec<WriteOperation>,
}

impl BatchDbWriter {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn drain(self) -> Vec<WriteOperation> {
        self.operations
    }
}

impl DbWriter for BatchDbWriter {
    fn put(&mut self, key: &[u8], value: &[u8]) -> StoreResult<()> {
        self.operations.push(WriteOperation::Put(key.to_vec(), value.to_vec()));
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> StoreResult<()> {
        self.operations.push(WriteOperation::Delete(key.to_vec()));
        Ok(())
    }
}

/// Thread-safe shared batch writer.
#[derive(Clone, Default)]
pub struct SharedBatchDbWriter {
    inner: Arc<Mutex<BatchDbWriter>>,
}

impl SharedBatchDbWriter {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(BatchDbWriter::new())),
        }
    }
}

impl DbWriter for SharedBatchDbWriter {
    fn put(&mut self, key: &[u8], value: &[u8]) -> StoreResult<()> {
        self.inner.lock().put(key, value)
    }

    fn delete(&mut self, key: &[u8]) -> StoreResult<()> {
        self.inner.lock().delete(key)
    }
}
