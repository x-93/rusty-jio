use crate::batch::BatchDbWriter;
use crate::cache::{Cache, CachePolicy};
use crate::db::DB;
use crate::error::DatabaseError;
use crate::key::DbKey;
use crate::registry::DatabaseStorePrefixes;
use std::sync::Arc;

/// Provides high-performance cached and prefixed access to a single table in RocksDB.
#[derive(Clone)]
pub struct CachedDbAccess {
    db: Arc<DB>,
    prefix: Vec<u8>,
    cache: Cache<Vec<u8>, Vec<u8>>,
}

impl CachedDbAccess {
    /// Create a new cached table accessor with LRU cache policy and default capacity.
    pub fn new(db: Arc<DB>, prefix: Vec<u8>, cache_capacity: usize) -> Self {
        Self::new_with_policy(db, prefix, CachePolicy::Lru, cache_capacity)
    }

    /// Create a new cached table accessor with specified eviction policy.
    pub fn new_with_policy(
        db: Arc<DB>,
        prefix: Vec<u8>,
        policy: CachePolicy,
        cache_capacity: usize,
    ) -> Self {
        Self {
            db,
            prefix,
            cache: Cache::new(policy, cache_capacity),
        }
    }

    /// Create from a known catalog prefix enum using LRU cache.
    pub fn from_store(db: Arc<DB>, store: DatabaseStorePrefixes, cache_capacity: usize) -> Self {
        Self::new(db, store.into(), cache_capacity)
    }

    /// Create from a known catalog prefix enum with specified eviction policy.
    pub fn from_store_with_policy(
        db: Arc<DB>,
        store: DatabaseStorePrefixes,
        policy: CachePolicy,
        cache_capacity: usize,
    ) -> Self {
        Self::new_with_policy(db, store.into(), policy, cache_capacity)
    }

    /// Access the table prefix bytes.
    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    /// Access the underlying cache.
    pub fn cache(&self) -> &Cache<Vec<u8>, Vec<u8>> {
        &self.cache
    }

    /// Check if a key exists either in the cache or on disk.
    pub fn has(&self, key: &[u8]) -> Result<bool, DatabaseError> {
        if self.cache.contains_key(&key.to_vec()) {
            return Ok(true);
        }
        let db_key = DbKey::new(&self.prefix, key);
        Ok(self.db.get(db_key.as_bytes())?.is_some())
    }

    /// Read a value by key. Checks cache first; on miss, queries RocksDB and warms cache.
    pub fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        let key_vec = key.to_vec();
        if let Some(val) = self.cache.get(&key_vec) {
            return Ok(Some(val));
        }

        let db_key = DbKey::new(&self.prefix, key);
        if let Some(val) = self.db.get(db_key.as_bytes())? {
            self.cache.insert(key_vec, val.clone());
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }

    /// Write a key-value pair to RocksDB and update the in-memory cache.
    pub fn write(&self, key: &[u8], value: Vec<u8>) -> Result<(), DatabaseError> {
        let db_key = DbKey::new(&self.prefix, key);
        self.db.put(db_key.as_bytes(), &value)?;
        self.cache.insert(key.to_vec(), value);
        Ok(())
    }

    /// Delete a key from RocksDB and remove from cache.
    pub fn delete(&self, key: &[u8]) -> Result<(), DatabaseError> {
        let db_key = DbKey::new(&self.prefix, key);
        self.db.delete(db_key.as_bytes())?;
        self.cache.remove(&key.to_vec());
        Ok(())
    }

    /// Record a write in a `BatchDbWriter` and update the local cache immediately.
    pub fn write_batch(&self, batch: &mut BatchDbWriter, key: &[u8], value: &[u8]) {
        batch.put_prefixed(&self.prefix, key, value);
        self.cache.insert(key.to_vec(), value.to_vec());
    }

    /// Record a delete in a `BatchDbWriter` and evict from the local cache immediately.
    pub fn delete_batch(&self, batch: &mut BatchDbWriter, key: &[u8]) {
        batch.delete_prefixed(&self.prefix, key);
        self.cache.remove(&key.to_vec());
    }
}
