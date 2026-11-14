use crate::prelude::{Cache, DB, DbKey, DbWriter, StoreError, StoreResult};
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::hash_map::RandomState, hash::BuildHasher, sync::Arc};

#[derive(Clone)]
pub struct CachedDbAccess<TKey, TData, S = RandomState>
where
    TKey: Clone + std::hash::Hash + Eq + Send + Sync + AsRef<[u8]>,
    TData: Clone + Send + Sync + Serialize + DeserializeOwned,
{
    db: Arc<DB>,
    cache: Cache<TKey, TData, S>,
    prefix: &'static [u8],
}

impl<TKey, TData, S> CachedDbAccess<TKey, TData, S>
where
    TKey: Clone + std::hash::Hash + Eq + Send + Sync + AsRef<[u8]>,
    TData: Clone + Send + Sync + Serialize + DeserializeOwned,
    S: BuildHasher + Default,
{
    pub fn new(db: Arc<DB>, cache_size: u64, prefix: &'static [u8]) -> Self {
        Self {
            db,
            cache: Cache::new(cache_size),
            prefix,
        }
    }

    pub fn has(&self, key: &TKey) -> StoreResult<bool> {
        if self.cache.contains_key(key) {
            return Ok(true);
        }
        let db_key = DbKey::new(self.prefix, key);
        Ok(self.db.get_pinned(db_key.as_ref())?.is_some())
    }

    pub fn get(&self, key: &TKey) -> StoreResult<TData> {
        if let Some(data) = self.cache.get(key) {
            return Ok(data);
        }
        let db_key = DbKey::new(self.prefix, key);
        let Some(bytes) = self.db.get_pinned(db_key.as_ref())? else {
            return Err(StoreError::KeyNotFound(db_key));
        };
        let data: TData = bincode::deserialize(&bytes)?;
        self.cache.insert(key.clone(), data.clone());
        Ok(data)
    }

    pub fn set(&self, writer: &mut impl DbWriter, key: TKey, data: TData) -> StoreResult<()> {
        let db_key = DbKey::new(self.prefix, &key);
        let bytes = bincode::serialize(&data)?;
        writer.put(db_key.as_ref(), &bytes)?;
        self.cache.insert(key, data);
        Ok(())
    }

    pub fn delete(&self, writer: &mut impl DbWriter, key: &TKey) -> StoreResult<()> {
        let db_key = DbKey::new(self.prefix, key);
        writer.delete(db_key.as_ref())?;
        self.cache.remove(key);
        Ok(())
    }

    pub fn iterator(&self) -> impl Iterator<Item = StoreResult<(TKey, TData)>> + '_
    where
        TKey: for<'a> TryFrom<&'a [u8]>,
    {
        let db_key = DbKey::prefix_only(self.prefix);
        let prefix_len = db_key.prefix_len();
        self.db
            .prefix_iterator(db_key.as_ref())
            .take_while(move |item| match item {
                Ok((k, _)) => k.starts_with(db_key.as_ref()),
                Err(_) => true,
            })
            .map(move |item| match item {
                Ok((k, v)) => {
                    let key_bytes = &k[prefix_len..];
                    let key = TKey::try_from(key_bytes).map_err(|_| {
                        StoreError::DeserializationError(Box::new(bincode::ErrorKind::Custom(
                            "invalid key".into(),
                        )))
                    })?;
                    let data: TData = bincode::deserialize(&v)?;
                    Ok((key, data))
                }
                Err(e) => Err(StoreError::DbError(e)),
            })
    }
}
