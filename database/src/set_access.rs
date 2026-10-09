//! Key-set and iteration storage access wrapper with caching.

use crate::{
    db::DB,
    errors::StoreError,
    prelude::{Cache, DbKey, DbWriter},
};
use parking_lot::RwLock;
use rocksdb::{Direction, IteratorMode, ReadOptions};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::{hash_map::RandomState, HashSet},
    error::Error,
    hash::{BuildHasher, Hash},
    sync::Arc,
};

/// Concurrent key-set storage access with caching.
#[derive(Clone)]
pub struct CachedDbSetAccess<TKey, TItem, S = RandomState>
where
    TKey: Clone + Hash + Eq + Send + Sync,
    TItem: Clone + Hash + Eq + Send + Sync,
{
    db: Arc<DB>,
    prefix: Vec<u8>,
    // Optional cache from key to HashSet of items
    cache: Cache<TKey, Arc<RwLock<HashSet<TItem>>>, S>,
}

impl<TKey, TItem, S> CachedDbSetAccess<TKey, TItem, S>
where
    TKey: Clone + Hash + Eq + Send + Sync,
    TItem: Clone + Hash + Eq + Send + Sync,
    S: BuildHasher + Default,
{
    pub fn new(db: Arc<DB>, cache_size: u64, prefix: Vec<u8>) -> Self {
        Self {
            db,
            prefix,
            cache: Cache::new(cache_size),
        }
    }

    /// Check if item belongs to the set for key.
    pub fn has(&self, key: TKey, item: TItem) -> Result<bool, StoreError>
    where
        TKey: AsRef<[u8]>,
        TItem: Serialize + DeserializeOwned,
    {
        if let Some(set_lock) = self.cache.get(&key) {
            return Ok(set_lock.read().contains(&item));
        }

        let item_bytes = bincode::serialize(&item)?;
        let mut composite_key = Vec::new();
        composite_key.extend_from_slice(key.as_ref());
        composite_key.push(b':');
        composite_key.extend_from_slice(&item_bytes);

        let db_key = DbKey::new(&self.prefix, &composite_key);
        Ok(self.db.get_pinned(&db_key)?.is_some())
    }

    /// Add an item to the key-set.
    pub fn write(&self, mut writer: impl DbWriter, key: TKey, item: TItem) -> Result<(), StoreError>
    where
        TKey: AsRef<[u8]>,
        TItem: Serialize,
    {
        if let Some(set_lock) = self.cache.get(&key) {
            set_lock.write().insert(item.clone());
        }

        let item_bytes = bincode::serialize(&item)?;
        let mut composite_key = Vec::new();
        composite_key.extend_from_slice(key.as_ref());
        composite_key.push(b':');
        composite_key.extend_from_slice(&item_bytes);

        let db_key = DbKey::new(&self.prefix, &composite_key);
        writer.put(db_key, [])?;
        Ok(())
    }

    /// Remove an item from the key-set.
    pub fn delete(&self, mut writer: impl DbWriter, key: TKey, item: TItem) -> Result<(), StoreError>
    where
        TKey: AsRef<[u8]>,
        TItem: Serialize,
    {
        if let Some(set_lock) = self.cache.get(&key) {
            set_lock.write().remove(&item);
        }

        let item_bytes = bincode::serialize(&item)?;
        let mut composite_key = Vec::new();
        composite_key.extend_from_slice(key.as_ref());
        composite_key.push(b':');
        composite_key.extend_from_slice(&item_bytes);

        let db_key = DbKey::new(&self.prefix, &composite_key);
        writer.delete(db_key)?;
        Ok(())
    }

    /// Read the full set of items associated with key.
    pub fn read_set(&self, key: TKey) -> Result<HashSet<TItem>, StoreError>
    where
        TKey: AsRef<[u8]>,
        TItem: DeserializeOwned,
    {
        if let Some(set_lock) = self.cache.get(&key) {
            return Ok(set_lock.read().clone());
        }

        let mut key_prefix = self.prefix.clone();
        key_prefix.push(b'/');
        key_prefix.extend_from_slice(key.as_ref());
        key_prefix.push(b':');

        let mut read_opts = ReadOptions::default();
        read_opts.set_iterate_range(rocksdb::PrefixRange(key_prefix.as_slice()));

        let mut items = HashSet::new();
        let iter = self
            .db
            .iterator_opt(IteratorMode::From(&key_prefix, Direction::Forward), read_opts);

        for res in iter {
            let (raw_key, _) = res?;
            if raw_key.starts_with(&key_prefix) {
                let item_bytes = &raw_key[key_prefix.len()..];
                let item: TItem = bincode::deserialize(item_bytes)?;
                items.insert(item);
            }
        }

        self.cache.insert(key, Arc::new(RwLock::new(items.clone())));
        Ok(items)
    }

    /// Iterator over raw key-set entries.
    pub fn raw_iterator(&self) -> impl Iterator<Item = Result<Box<[u8]>, Box<dyn Error>>> + '_ {
        let db_key = DbKey::prefix_only(&self.prefix);
        let mut read_opts = ReadOptions::default();
        read_opts.set_iterate_range(rocksdb::PrefixRange(db_key.as_ref()));
        self.db
            .iterator_opt(IteratorMode::From(db_key.as_ref(), Direction::Forward), read_opts)
            .map(|res| match res {
                Ok((key, _)) => Ok(key),
                Err(e) => Err(e.into()),
            })
    }
}
