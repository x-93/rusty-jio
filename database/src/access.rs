use crate::errors::{StoreError, StoreResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait DirectDbAccess<K, V> {
    fn get(&self, key: &K) -> StoreResult<V>;
    fn has(&self, key: &K) -> bool;
    fn set(&self, key: K, value: V) -> StoreResult<()>;
    fn remove(&self, key: &K) -> StoreResult<()>;
}

#[derive(Clone, Default)]
pub struct MemoryAccess<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    map: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> MemoryAccess<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K, V> DirectDbAccess<K, V> for MemoryAccess<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, key: &K) -> StoreResult<V> {
        self.map
            .read()
            .get(key)
            .cloned()
            .ok_or_else(|| StoreError::KeyNotFound("key not found in memory store".to_string()))
    }

    fn has(&self, key: &K) -> bool {
        self.map.read().contains_key(key)
    }

    fn set(&self, key: K, value: V) -> StoreResult<()> {
        self.map.write().insert(key, value);
        Ok(())
    }

    fn remove(&self, key: &K) -> StoreResult<()> {
        self.map.write().remove(key);
        Ok(())
    }
}
