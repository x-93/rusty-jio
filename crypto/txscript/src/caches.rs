use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default, Clone, Debug)]
pub struct TxScriptCache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + Clone,
    V: Clone,
{
    cache: Arc<RwLock<HashMap<K, V>>>,
}

impl<K, V> TxScriptCache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.cache.read().get(key).cloned()
    }

    pub fn insert(&self, key: K, value: V) {
        self.cache.write().insert(key, value);
    }

    pub fn clear(&self) {
        self.cache.write().clear();
    }
}
