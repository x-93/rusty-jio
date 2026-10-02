//! Cached database item for singleton values.

use std::sync::Arc;
use parking_lot::RwLock;
use borsh::{BorshDeserialize, BorshSerialize};

use super::db::DB;
use super::errors::{StoreError, StoreResult};
use super::key::DbKey;
use super::utils::{deserialize_from_slice, serialize_to_vec};
use super::writer::DbWriter;

pub struct CachedDbItem<TData>
where
    TData: Clone + BorshSerialize + BorshDeserialize,
{
    db: Arc<DB>,
    key: DbKey,
    cache: RwLock<Option<Option<TData>>>,
}

impl<TData> CachedDbItem<TData>
where
    TData: Clone + BorshSerialize + BorshDeserialize,
{
    pub fn new(db: Arc<DB>, prefix: &'static [u8]) -> Self {
        Self {
            db,
            key: DbKey::new(prefix, &[]),
            cache: RwLock::new(None),
        }
    }

    pub fn get(&self) -> StoreResult<TData> {
        {
            let guard = self.cache.read();
            if let Some(cached) = &*guard {
                return match cached {
                    Some(val) => Ok(val.clone()),
                    None => Err(StoreError::KeyNotFound(format!("{:?}", self.key))),
                };
            }
        }

        let raw_opt = self.db.get_raw(self.key.as_bytes())?;
        match raw_opt {
            Some(raw) => {
                let data: TData = deserialize_from_slice(&raw)?;
                let mut guard = self.cache.write();
                *guard = Some(Some(data.clone()));
                Ok(data)
            }
            None => {
                let mut guard = self.cache.write();
                *guard = Some(None);
                Err(StoreError::KeyNotFound(format!("{:?}", self.key)))
            }
        }
    }

    pub fn set(&self, mut writer: impl DbWriter, data: TData) -> StoreResult<()> {
        let bytes = serialize_to_vec(&data)?;
        writer.put(self.key.as_bytes(), &bytes)?;
        let mut guard = self.cache.write();
        *guard = Some(Some(data));
        Ok(())
    }

    pub fn delete(&self, mut writer: impl DbWriter) -> StoreResult<()> {
        writer.delete(self.key.as_bytes())?;
        let mut guard = self.cache.write();
        *guard = Some(None);
        Ok(())
    }
}
