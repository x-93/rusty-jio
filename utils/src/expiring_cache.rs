//! Expiring cache with time-to-live eviction.

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub struct ExpiringCache<K, V> {
    map: HashMap<K, (V, Instant)>,
    ttl: Duration,
}

impl<K: Eq + Hash + Clone, V: Clone> ExpiringCache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: HashMap::new(),
            ttl,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, (value, Instant::now()));
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((_, timestamp)) = self.map.get(key) {
            if timestamp.elapsed() > self.ttl {
                self.map.remove(key);
                return None;
            }
        }
        self.map.get(key).map(|(v, _)| v)
    }

    pub fn contains_key(&mut self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn prune(&mut self) {
        let ttl = self.ttl;
        self.map.retain(|_, (_, timestamp)| timestamp.elapsed() <= ttl);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
