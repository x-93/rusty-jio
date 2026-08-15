use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ExpiringCache<K, V> {
    store: HashMap<K, (V, Instant)>,
    ttl: Duration,
}

impl<K: Eq + Hash + Clone, V: Clone> ExpiringCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            store: HashMap::new(),
            ttl,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.store.insert(key, (value, Instant::now()));
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let (val, timestamp) = self.store.get(key)?;
        if timestamp.elapsed() > self.ttl {
            None
        } else {
            Some(val.clone())
        }
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.store.remove(key).map(|(v, _)| v)
    }

    pub fn clean_expired(&mut self) {
        let now = Instant::now();
        let ttl = self.ttl;
        self.store.retain(|_, (_, ts)| now.duration_since(*ts) <= ttl);
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expiring_cache() {
        let mut cache = ExpiringCache::new(Duration::from_millis(50));
        cache.insert("key1", "val1");
        assert_eq!(cache.get(&"key1"), Some("val1"));
    }
}
