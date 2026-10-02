//! Cached database set access for tracking collections of keys.

use std::collections::HashSet;
use std::hash::Hash as StdHash;
use std::sync::Arc;
use parking_lot::RwLock;

use super::access::KeySerializer;
use super::db::DB;
use super::errors::StoreResult;
use super::key::DbKey;
use super::writer::DbWriter;

pub struct CachedDbSetAccess<TKey>
where
    TKey: Clone + Eq + StdHash + KeySerializer,
{
    db: Arc<DB>,
    prefix: &'static [u8],
    cache: RwLock<HashSet<TKey>>,
}

impl<TKey> CachedDbSetAccess<TKey>
where
    TKey: Clone + Eq + StdHash + KeySerializer,
{
    pub fn new(db: Arc<DB>, prefix: &'static [u8]) -> Self {
        Self {
            db,
            prefix,
            cache: RwLock::new(HashSet::new()),
        }
    }

    fn to_db_key(&self, key: &TKey) -> DbKey {
        DbKey::new(self.prefix, &key.serialize_key())
    }

    pub fn has(&self, key: &TKey) -> StoreResult<bool> {
        let guard = self.cache.read();
        if guard.contains(key) {
            return Ok(true);
        }
        let db_key = self.to_db_key(key);
        self.db.has_raw(db_key.as_bytes())
    }

    pub fn insert(&self, mut writer: impl DbWriter, key: TKey) -> StoreResult<()> {
        let db_key = self.to_db_key(&key);
        writer.put(db_key.as_bytes(), &[])?;
        let mut guard = self.cache.write();
        guard.insert(key);
        Ok(())
    }

    pub fn remove(&self, mut writer: impl DbWriter, key: &TKey) -> StoreResult<()> {
        let db_key = self.to_db_key(key);
        writer.delete(db_key.as_bytes())?;
        let mut guard = self.cache.write();
        guard.remove(key);
        Ok(())
    }
}
