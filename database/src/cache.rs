use parking_lot::RwLock;
use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    /// Least Recently Used eviction. Mutates access ordering on reads.
    Lru,
    /// Random eviction on overflow. Reads do not mutate ordering, allowing lock contention reduction.
    Random,
}

/// A node in the LRU doubly linked list.
struct LruNode<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

struct LruCache<K, V> {
    map: HashMap<K, usize>,
    nodes: Vec<Option<LruNode<K, V>>>,
    free_slots: Vec<usize>,
    head: Option<usize>, // Most recently used
    tail: Option<usize>, // Least recently used
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            free_slots: Vec::new(),
            head: None,
            tail: None,
            capacity,
        }
    }

    fn get(&mut self, key: &K) -> Option<V> {
        let &idx = self.map.get(key)?;
        self.move_to_head(idx);
        self.nodes[idx].as_ref().map(|node| node.value.clone())
    }

    fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.capacity == 0 {
            return None;
        }

        if let Some(&idx) = self.map.get(&key) {
            let old_val = self.nodes[idx].as_mut().map(|node| {
                let old = node.value.clone();
                node.value = value;
                old
            });
            self.move_to_head(idx);
            return old_val;
        }

        if self.map.len() >= self.capacity {
            self.evict_tail();
        }

        let idx = if let Some(slot) = self.free_slots.pop() {
            self.nodes[slot] = Some(LruNode {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            });
            slot
        } else {
            let slot = self.nodes.len();
            self.nodes.push(Some(LruNode {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            }));
            slot
        };

        self.attach_head(idx);
        self.map.insert(key, idx);
        None
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let idx = self.map.remove(key)?;
        self.detach(idx);
        self.free_slots.push(idx);
        self.nodes[idx].take().map(|node| node.value)
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn clear(&mut self) {
        self.map.clear();
        self.nodes.clear();
        self.free_slots.clear();
        self.head = None;
        self.tail = None;
    }

    fn detach(&mut self, idx: usize) {
        let (prev, next) = match &self.nodes[idx] {
            Some(node) => (node.prev, node.next),
            None => return,
        };

        if let Some(p) = prev {
            if let Some(Some(prev_node)) = self.nodes.get_mut(p) {
                prev_node.next = next;
            }
        } else {
            self.head = next;
        }

        if let Some(n) = next {
            if let Some(Some(next_node)) = self.nodes.get_mut(n) {
                next_node.prev = prev;
            }
        } else {
            self.tail = prev;
        }

        if let Some(Some(node)) = self.nodes.get_mut(idx) {
            node.prev = None;
            node.next = None;
        }
    }

    fn attach_head(&mut self, idx: usize) {
        let old_head = self.head;
        if let Some(Some(node)) = self.nodes.get_mut(idx) {
            node.prev = None;
            node.next = old_head;
        }

        if let Some(h) = old_head {
            if let Some(Some(head_node)) = self.nodes.get_mut(h) {
                head_node.prev = Some(idx);
            }
        } else {
            self.tail = Some(idx);
        }

        self.head = Some(idx);
    }

    fn move_to_head(&mut self, idx: usize) {
        if self.head == Some(idx) {
            return;
        }
        self.detach(idx);
        self.attach_head(idx);
    }

    fn evict_tail(&mut self) {
        if let Some(tail_idx) = self.tail {
            self.detach(tail_idx);
            self.free_slots.push(tail_idx);
            if let Some(node) = self.nodes[tail_idx].take() {
                self.map.remove(&node.key);
            }
        }
    }
}

struct RandomCache<K, V> {
    map: HashMap<K, (V, usize)>,
    keys: Vec<K>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> RandomCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            keys: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn get(&self, key: &K) -> Option<V> {
        self.map.get(key).map(|(v, _)| v.clone())
    }

    fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.capacity == 0 {
            return None;
        }

        if let Some((old_val, _)) = self.map.get_mut(&key) {
            let prev = old_val.clone();
            *old_val = value;
            return Some(prev);
        }

        if self.keys.len() >= self.capacity {
            let victim_idx = rand::thread_rng().gen_range(0..self.keys.len());
            let victim_key = self.keys[victim_idx].clone();

            let last_idx = self.keys.len() - 1;
            if victim_idx != last_idx {
                self.keys.swap(victim_idx, last_idx);
                let moved_key = &self.keys[victim_idx];
                if let Some((_, slot)) = self.map.get_mut(moved_key) {
                    *slot = victim_idx;
                }
            }
            self.keys.pop();
            self.map.remove(&victim_key);
        }

        let new_idx = self.keys.len();
        self.keys.push(key.clone());
        self.map.insert(key, (value, new_idx));
        None
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let (val, idx) = self.map.remove(key)?;
        let last_idx = self.keys.len() - 1;
        if idx != last_idx {
            self.keys.swap(idx, last_idx);
            let moved_key = &self.keys[idx];
            if let Some((_, slot)) = self.map.get_mut(moved_key) {
                *slot = idx;
            }
        }
        self.keys.pop();
        Some(val)
    }

    fn len(&self) -> usize {
        self.keys.len()
    }

    fn clear(&mut self) {
        self.map.clear();
        self.keys.clear();
    }
}

enum CacheInner<K, V> {
    Lru(LruCache<K, V>),
    Random(RandomCache<K, V>),
}

/// A thread-safe in-memory cache supporting either LRU or Random eviction policies.
#[derive(Clone)]
pub struct Cache<K, V> {
    inner: Arc<RwLock<CacheInner<K, V>>>,
    policy: CachePolicy,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> Cache<K, V> {
    /// Create a cache using LRU eviction.
    pub fn new_lru(capacity: usize) -> Self {
        Self::new(CachePolicy::Lru, capacity)
    }

    /// Create a cache using Random eviction.
    pub fn new_random(capacity: usize) -> Self {
        Self::new(CachePolicy::Random, capacity)
    }

    /// Create a cache with specified policy and capacity.
    pub fn new(policy: CachePolicy, capacity: usize) -> Self {
        let inner = match policy {
            CachePolicy::Lru => CacheInner::Lru(LruCache::new(capacity)),
            CachePolicy::Random => CacheInner::Random(RandomCache::new(capacity)),
        };
        Self {
            inner: Arc::new(RwLock::new(inner)),
            policy,
            capacity,
        }
    }

    pub fn policy(&self) -> CachePolicy {
        self.policy
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        match &*self.inner.read() {
            CacheInner::Lru(c) => c.len(),
            CacheInner::Random(c) => c.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains_key(&self, key: &K) -> bool {
        match &*self.inner.read() {
            CacheInner::Lru(c) => c.contains_key(key),
            CacheInner::Random(c) => c.contains_key(key),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        match self.policy {
            CachePolicy::Lru => {
                // LRU needs write lock to update MRU position
                let mut guard = self.inner.write();
                if let CacheInner::Lru(c) = &mut *guard {
                    c.get(key)
                } else {
                    None
                }
            }
            CachePolicy::Random => {
                // Random cache only requires read lock!
                let guard = self.inner.read();
                if let CacheInner::Random(c) = &*guard {
                    c.get(key)
                } else {
                    None
                }
            }
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut guard = self.inner.write();
        match &mut *guard {
            CacheInner::Lru(c) => c.insert(key, value),
            CacheInner::Random(c) => c.insert(key, value),
        }
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut guard = self.inner.write();
        match &mut *guard {
            CacheInner::Lru(c) => c.remove(key),
            CacheInner::Random(c) => c.remove(key),
        }
    }

    pub fn clear(&self) {
        let mut guard = self.inner.write();
        match &mut *guard {
            CacheInner::Lru(c) => c.clear(),
            CacheInner::Random(c) => c.clear(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_eviction() {
        let cache = Cache::<u32, &'static str>::new_lru(3);
        cache.insert(1, "one");
        cache.insert(2, "two");
        cache.insert(3, "three");

        assert_eq!(cache.len(), 3);
        // Access 1 so it becomes most recently used
        assert_eq!(cache.get(&1), Some("one"));

        // Insert 4: 2 should be evicted because 1 was accessed and 3 was inserted after 2
        cache.insert(4, "four");
        assert_eq!(cache.len(), 3);

        assert_eq!(cache.get(&1), Some("one"));
        assert_eq!(cache.get(&2), None); // evicted!
        assert_eq!(cache.get(&3), Some("three"));
        assert_eq!(cache.get(&4), Some("four"));
    }

    #[test]
    fn test_random_cache_bounds() {
        let cache = Cache::<u32, u32>::new_random(10);
        for i in 0..50 {
            cache.insert(i, i * 10);
            assert!(cache.len() <= 10);
        }
        assert_eq!(cache.len(), 10);
    }

    #[test]
    fn test_remove_and_clear() {
        let cache = Cache::<String, u32>::new_lru(5);
        cache.insert("a".into(), 1);
        cache.insert("b".into(), 2);

        assert_eq!(cache.remove(&"a".into()), Some(1));
        assert_eq!(cache.get(&"a".into()), None);
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}

