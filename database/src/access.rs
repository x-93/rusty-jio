//! Cached typed database access with prefix scoping and cache eviction.

use std::collections::HashMap;
use std::hash::Hash as StdHash;
use std::sync::Arc;
use parking_lot::RwLock;
use borsh::{BorshDeserialize, BorshSerialize};

use super::cache::CachePolicy;
use super::db::DB;
use super::errors::{StoreError, StoreResult};
use super::key::DbKey;
use super::utils::{deserialize_from_slice, serialize_to_vec};
use super::writer::DbWriter;

pub trait KeySerializer: Send + Sync {
    fn serialize_key(&self) -> Vec<u8>;
}

impl<T: AsRef<[u8]> + Send + Sync> KeySerializer for T {
    fn serialize_key(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}

pub struct CachedDbAccess<TKey, TData>
where
    TKey: Clone + Eq + StdHash + KeySerializer,
    TData: Clone + BorshSerialize + BorshDeserialize,
{
    db: Arc<DB>,
    prefix: &'static [u8],
    cache: RwLock<HashMap<TKey, Option<TData>>>,
    policy: CachePolicy,
}

impl<TKey, TData> CachedDbAccess<TKey, TData>
where
    TKey: Clone + Eq + StdHash + KeySerializer,
    TData: Clone + BorshSerialize + BorshDeserialize,
{
    pub fn new(db: Arc<DB>, prefix: &'static [u8], policy: CachePolicy) -> Self {
        Self {
            db,
            prefix,
            cache: RwLock::new(HashMap::new()),
            policy,
        }
    }

    fn to_db_key(&self, key: &TKey) -> DbKey {
        DbKey::new(self.prefix, &key.serialize_key())
    }

    pub fn get(&self, key: &TKey) -> StoreResult<TData> {
        if matches!(self.policy, CachePolicy::Count(_)) {
            let guard = self.cache.read();
            if let Some(cached) = guard.get(key) {
                return match cached {
                    Some(val) => Ok(val.clone()),
                    None => Err(StoreError::KeyNotFound(format!("{:?}", self.prefix))),
                };
            }
        }

        let db_key = self.to_db_key(key);
        let raw_opt = self.db.get_raw(db_key.as_bytes())?;

        match raw_opt {
            Some(raw) => {
                let data: TData = deserialize_from_slice(&raw)?;
                if matches!(self.policy, CachePolicy::Count(_)) {
                    let mut guard = self.cache.write();
                    if let CachePolicy::Count(limit) = self.policy {
                        if guard.len() >= limit {
                            guard.clear();
                        }
                    }
                    guard.insert(key.clone(), Some(data.clone()));
                }
                Ok(data)
            }
            None => {
                if matches!(self.policy, CachePolicy::Count(_)) {
                    let mut guard = self.cache.write();
                    guard.insert(key.clone(), None);
                }
                Err(StoreError::KeyNotFound(format!("{:?}", self.prefix)))
            }
        }
    }

    pub fn has(&self, key: &TKey) -> StoreResult<bool> {
        if matches!(self.policy, CachePolicy::Count(_)) {
            let guard = self.cache.read();
            if let Some(cached) = guard.get(key) {
                return Ok(cached.is_some());
            }
        }

        let db_key = self.to_db_key(key);
        self.db.has_raw(db_key.as_bytes())
    }

    pub fn set(&self, mut writer: impl DbWriter, key: TKey, data: TData) -> StoreResult<()> {
        let db_key = self.to_db_key(&key);
        let bytes = serialize_to_vec(&data)?;
        writer.put(db_key.as_bytes(), &bytes)?;

        if matches!(self.policy, CachePolicy::Count(_)) {
            let mut guard = self.cache.write();
            guard.insert(key, Some(data));
        }
        Ok(())
    }

    pub fn delete(&self, mut writer: impl DbWriter, key: &TKey) -> StoreResult<()> {
        let db_key = self.to_db_key(key);
        writer.delete(db_key.as_bytes())?;

        if matches!(self.policy, CachePolicy::Count(_)) {
            let mut guard = self.cache.write();
            guard.insert(key.clone(), None);
        }
        Ok(())
    }
}
