use jio_consensus_core::blockhash::{self, BlockHash};
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
        Self {
            start: 1,
            end: u64::MAX,
        }
    }

    #[inline]
    pub const fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    #[inline]
    pub const fn size(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.start == 0 && self.end == 0
    }
}

pub trait ReachabilityStoreReader {
    fn get_interval(&self, hash: Hash) -> Option<Interval>;
    fn get_parent(&self, hash: Hash) -> Option<BlockHash>;
    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>>;
    fn has(&self, hash: Hash) -> Option<bool>;
}

/// Unsynchronized in-memory reachability store for single-threaded or test use
#[derive(Default, Clone, Debug)]
pub struct MemoryReachabilityStore {
    intervals: HashMap<Hash, Interval>,
    parents: HashMap<Hash, BlockHash>,
    children: HashMap<Hash, Arc<Vec<BlockHash>>>,
}

impl MemoryReachabilityStore {
    pub fn new() -> Self {
        Self {
            intervals: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }

    pub fn insert(&mut self, hash: Hash, parent: Hash, interval: Interval) {
        self.intervals.insert(hash, interval);
        self.parents.insert(hash, parent);
        let children_entry = self
            .children
            .entry(parent)
            .or_insert_with(|| Arc::new(Vec::new()));
        let mut current = (**children_entry).clone();
        if !current.contains(&hash) {
            current.push(hash);
            *children_entry = Arc::new(current);
        }
    }

    pub fn set_interval(&mut self, hash: Hash, interval: Interval) {
        self.intervals.insert(hash, interval);
    }

    pub fn delete(&mut self, hash: Hash) {
        self.intervals.remove(&hash);
        self.parents.remove(&hash);
        self.children.remove(&hash);
    }
}

impl ReachabilityStoreReader for MemoryReachabilityStore {
    fn get_interval(&self, hash: Hash) -> Option<Interval> {
        self.intervals.get(&hash).copied()
    }

    fn get_parent(&self, hash: Hash) -> Option<BlockHash> {
        self.parents.get(&hash).copied()
    }

    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>> {
        self.children.get(&hash).cloned()
    }

    fn has(&self, hash: Hash) -> Option<bool> {
        Some(self.intervals.contains_key(&hash))
    }
}

/// Synchronized in-memory reachability store
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
        let mut current = children_map
            .get(&parent)
            .map(|v| (**v).clone())
            .unwrap_or_default();
        if !current.contains(&hash) {
            current.push(hash);
            children_map.insert(parent, Arc::new(current));
        }
    }

    pub fn init_genesis(&self, genesis: BlockHash) {
        self.insert(genesis, blockhash::ORIGIN, Interval::maximal());
    }

    pub fn add_block(&self, hash: BlockHash, selected_parent: BlockHash) {
        let parent_interval = self.get_interval(&selected_parent).unwrap_or(Interval::maximal());
        let existing_children_count = self
            .get_children(&selected_parent)
            .map(|c| c.len())
            .unwrap_or(0) as u64;

        let parent_size = parent_interval.size();
        let chunk_size = (parent_size / 65536).max(1);

        let child_start = parent_interval
            .start
            .saturating_add(existing_children_count.saturating_mul(chunk_size))
            .saturating_add(1);
        let child_end = child_start
            .saturating_add(chunk_size)
            .min(parent_interval.end.saturating_sub(1));

        let child_interval = if child_start < child_end {
            Interval::new(child_start, child_end)
        } else {
            Interval::new(
                parent_interval.start.saturating_add(1),
                parent_interval.end.saturating_sub(1),
            )
        };

        self.insert(hash, selected_parent, child_interval);
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
        <Self as ReachabilityStoreReader>::has(self, *hash).unwrap_or(false)
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

    fn has(&self, hash: Hash) -> Option<bool> {
        Some(self.intervals.read().contains_key(&hash))
    }
}
