use crate::prelude::{DB, DbKey, DbWriter, StoreError, StoreResult};
use parking_lot::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

/// Cached access for a single database item (such as DAG sink, pruning point, or virtual state).
#[derive(Clone)]
pub struct CachedDbItem<TData>
where
    TData: Clone + Send + Sync + Serialize + DeserializeOwned,
{
    db: Arc<DB>,
    key: DbKey,
    cached: Arc<RwLock<Option<TData>>>,
}

impl<TData> CachedDbItem<TData>
where
    TData: Clone + Send + Sync + Serialize + DeserializeOwned,
{
    pub fn new(db: Arc<DB>, prefix: &'static [u8]) -> Self {
        Self {
            db,
            key: DbKey::prefix_only(prefix),
            cached: Arc::new(RwLock::new(None)),
        }
    }

    pub fn has(&self) -> StoreResult<bool> {
        if self.cached.read().is_some() {
            return Ok(true);
        }
        Ok(self.db.get_pinned(self.key.as_ref())?.is_some())
    }

    pub fn get(&self) -> StoreResult<TData> {
        if let Some(data) = self.cached.read().as_ref() {
            return Ok(data.clone());
        }
        let Some(bytes) = self.db.get_pinned(self.key.as_ref())? else {
            return Err(StoreError::KeyNotFound(self.key.clone()));
        };
        let data: TData = bincode::deserialize(&bytes)?;
        *self.cached.write() = Some(data.clone());
        Ok(data)
    }

    pub fn set(&self, writer: &mut impl DbWriter, data: TData) -> StoreResult<()> {
        let bytes = bincode::serialize(&data)?;
        writer.put(self.key.as_ref(), &bytes)?;
        *self.cached.write() = Some(data);
        Ok(())
    }

    pub fn remove(&self, writer: &mut impl DbWriter) -> StoreResult<()> {
        writer.delete(self.key.as_ref())?;
        *self.cached.write() = None;
        Ok(())
    }
}
