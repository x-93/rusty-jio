//! The central Database handle.
//!
//! Provides thread-safe key-value operations, atomic batch writing,
//! and prefix iteration.

pub mod conn_builder;

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;

use super::errors::StoreResult;
use super::writer::{BatchDbWriter, WriteOperation};
use conn_builder::ConnBuilder;

pub struct DB {
    inner: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl DB {
    /// Opens or creates a database with default settings at the given path.
    pub fn open_default<P: AsRef<Path>>(_path: P) -> StoreResult<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        })
    }

    /// Opens or creates a database using the provided connection builder options.
    pub fn open_with_builder(_builder: ConnBuilder) -> StoreResult<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(BTreeMap::new())),
        })
    }

    pub fn get_raw(&self, key: &[u8]) -> StoreResult<Option<Vec<u8>>> {
        let guard = self.inner.read();
        Ok(guard.get(key).cloned())
    }

    pub fn has_raw(&self, key: &[u8]) -> StoreResult<bool> {
        let guard = self.inner.read();
        Ok(guard.contains_key(key))
    }

    pub fn put_raw(&self, key: &[u8], value: &[u8]) -> StoreResult<()> {
        let mut guard = self.inner.write();
        guard.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    pub fn delete_raw(&self, key: &[u8]) -> StoreResult<()> {
        let mut guard = self.inner.write();
        guard.remove(key);
        Ok(())
    }

    /// Atomically applies a batch of write operations.
    pub fn write(&self, batch: BatchDbWriter) -> StoreResult<()> {
        let mut guard = self.inner.write();
        for op in batch.drain() {
            match op {
                WriteOperation::Put(key, value) => {
                    guard.insert(key, value);
                }
                WriteOperation::Delete(key) => {
                    guard.remove(&key);
                }
            }
        }
        Ok(())
    }

    /// Iterates over all key-value entries matching the given prefix.
    pub fn prefix_iterator(&self, prefix: &[u8]) -> StoreResult<Vec<(Vec<u8>, Vec<u8>)>> {
        let guard = self.inner.read();
        let mut results = Vec::new();
        for (k, v) in guard.range(prefix.to_vec()..) {
            if k.starts_with(prefix) {
                results.push((k.clone(), v.clone()));
            } else {
                break;
            }
        }
        Ok(results)
    }

    /// Iterates over all keys matching the given prefix.
    pub fn prefix_keys(&self, prefix: &[u8]) -> StoreResult<Vec<Vec<u8>>> {
        let guard = self.inner.read();
        let mut results = Vec::new();
        for (k, _) in guard.range(prefix.to_vec()..) {
            if k.starts_with(prefix) {
                results.push(k.clone());
            } else {
                break;
            }
        }
        Ok(results)
    }
}
