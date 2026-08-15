use jio_consensus_core::blockhash::BlockHash;
use jio_hashes::Hash;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Interval {
    pub start: u64,
    pub end: u64,
}

impl Interval {
    pub const fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    pub const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    pub const fn maximal() -> Self {
        Self { start: 1, end: u64::MAX }
    }

    #[inline]
    pub const fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    #[inline]
    pub const fn size(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }
}

pub trait ReachabilityStoreReader {
    fn get_interval(&self, hash: Hash) -> Option<Interval>;
    fn get_parent(&self, hash: Hash) -> Option<BlockHash>;
    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>>;
    fn has(&self, hash: Hash) -> bool;
}

#[derive(Default, Clone)]
pub struct ReachabilityStore {
    intervals: Arc<RwLock<HashMap<Hash, Interval>>>,
    parents: Arc<RwLock<HashMap<Hash, BlockHash>>>,
    children: Arc<RwLock<HashMap<Hash, Arc<Vec<BlockHash>>>>>,
}

impl ReachabilityStore {
    pub fn new() -> Self {
        Self {
            intervals: Arc::new(RwLock::new(HashMap::new())),
            parents: Arc::new(RwLock::new(HashMap::new())),
            children: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, parent: Hash, interval: Interval) {
        self.intervals.write().insert(hash, interval);
        self.parents.write().insert(hash, parent);
        let mut children_map = self.children.write();
        let mut current = children_map.get(&parent).map(|v| (**v).clone()).unwrap_or_default();
        current.push(hash);
        children_map.insert(parent, Arc::new(current));
    }

    pub fn set_interval(&self, hash: Hash, interval: Interval) {
        self.intervals.write().insert(hash, interval);
    }

    pub fn delete(&self, hash: Hash) {
        self.intervals.write().remove(&hash);
        self.parents.write().remove(&hash);
        self.children.write().remove(&hash);
    }

    pub fn get_interval(&self, hash: &BlockHash) -> Option<Interval> {
        <Self as ReachabilityStoreReader>::get_interval(self, *hash)
    }

    pub fn get_parent(&self, hash: &BlockHash) -> Option<BlockHash> {
        <Self as ReachabilityStoreReader>::get_parent(self, *hash)
    }

    pub fn get_children(&self, hash: &BlockHash) -> Option<Vec<BlockHash>> {
        <Self as ReachabilityStoreReader>::get_children(self, *hash).map(|v| (*v).clone())
    }

    pub fn has(&self, hash: &BlockHash) -> bool {
        <Self as ReachabilityStoreReader>::has(self, *hash)
    }
}

impl ReachabilityStoreReader for ReachabilityStore {
    fn get_interval(&self, hash: Hash) -> Option<Interval> {
        self.intervals.read().get(&hash).copied()
    }

    fn get_parent(&self, hash: Hash) -> Option<BlockHash> {
        self.parents.read().get(&hash).copied()
    }

    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>> {
        self.children.read().get(&hash).cloned()
    }

    fn has(&self, hash: Hash) -> bool {
        self.intervals.read().contains_key(&hash)
    }
}
