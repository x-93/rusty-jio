use parking_lot::RwLock;
use std::collections::HashMap;

pub struct StoreCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    cache: RwLock<HashMap<K, V>>,
    capacity: usize,
}

impl<K, V> StoreCache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.cache.read().get(key).cloned()
    }

    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.write();
        if cache.len() >= self.capacity {
            // Simple eviction of first entry if capacity exceeded
            if let Some(first_key) = cache.keys().next().cloned() {
                cache.remove(&first_key);
            }
        }
        cache.insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.cache.write().remove(key)
    }

    pub fn clear(&self) {
        self.cache.write().clear();
    }
}
