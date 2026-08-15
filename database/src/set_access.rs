use crate::errors::StoreResult;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;

pub trait SetDbAccess<T> {
    fn insert(&self, item: T) -> StoreResult<bool>;
    fn remove(&self, item: &T) -> StoreResult<bool>;
    fn contains(&self, item: &T) -> bool;
    fn get_all(&self) -> Vec<T>;
}

#[derive(Clone, Default)]
pub struct MemorySetAccess<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    set: Arc<RwLock<HashSet<T>>>,
}

impl<T> MemorySetAccess<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            set: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

impl<T> SetDbAccess<T> for MemorySetAccess<T>
where
    T: std::hash::Hash + Eq + Clone + Send + Sync,
{
    fn insert(&self, item: T) -> StoreResult<bool> {
        Ok(self.set.write().insert(item))
    }

    fn remove(&self, item: &T) -> StoreResult<bool> {
        Ok(self.set.write().remove(item))
    }

    fn contains(&self, item: &T) -> bool {
        self.set.read().contains(item)
    }

    fn get_all(&self) -> Vec<T> {
        self.set.read().iter().cloned().collect()
    }
}
