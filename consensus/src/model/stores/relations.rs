use jio_consensus_core::blockhash::{BlockHash, BlockHashes};
use jio_consensus_core::{BlockHashSet, HashMapCustomHasher};
use jio_database::prelude::{ReadLock, StoreError, StoreResult};
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait RelationsStoreReader {
    fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError>;
    fn get_children(&self, hash: Hash) -> StoreResult<ReadLock<BlockHashSet>>;
    fn has(&self, hash: Hash) -> Result<bool, StoreError>;
    fn counts(&self) -> Result<(usize, usize), StoreError>;
}

/// Unsynchronized memory relations store
#[derive(Default, Clone, Debug)]
pub struct MemoryRelationsStore {
    parents: HashMap<Hash, BlockHashes>,
    children: HashMap<Hash, Arc<BlockHashSet>>,
}

impl MemoryRelationsStore {
    pub fn new() -> Self {
        Self {
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }

    pub fn insert(&mut self, hash: Hash, parents: BlockHashes) {
        for &parent in parents.iter() {
            let entry = self
                .children
                .entry(parent)
                .or_insert_with(|| Arc::new(BlockHashSet::new()));
            let mut current = (**entry).clone();
            current.insert(hash);
            *entry = Arc::new(current);
        }
        self.parents.insert(hash, parents);
    }

    pub fn delete(&mut self, hash: Hash) {
        if let Some(parents) = self.parents.remove(&hash) {
            for parent in parents.iter() {
                if let Some(entry) = self.children.get_mut(parent) {
                    let mut current = (**entry).clone();
                    current.remove(&hash);
                    *entry = Arc::new(current);
                }
            }
        }
        self.children.remove(&hash);
    }
}

impl RelationsStoreReader for MemoryRelationsStore {
    fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        self.parents
            .get(&hash)
            .cloned()
            .ok_or_else(|| StoreError::KeyNotFound(format!("relations parents for {hash} not found")))
    }

    fn get_children(&self, hash: Hash) -> StoreResult<ReadLock<BlockHashSet>> {
        self.children
            .get(&hash)
            .cloned()
            .ok_or_else(|| StoreError::KeyNotFound(format!("relations children for {hash} not found")))
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        Ok(self.parents.contains_key(&hash))
    }

    fn counts(&self) -> Result<(usize, usize), StoreError> {
        Ok((self.parents.len(), self.children.len()))
    }
}

/// Synchronized memory relations store
#[derive(Default, Clone)]
pub struct RelationsStore {
    parents: Arc<RwLock<HashMap<Hash, BlockHashes>>>,
    children: Arc<RwLock<HashMap<Hash, Arc<BlockHashSet>>>>,
}

impl RelationsStore {
    pub fn new() -> Self {
        Self {
            parents: Arc::new(RwLock::new(HashMap::new())),
            children: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, parents: Vec<BlockHash>) {
        let parents_arc = Arc::new(parents);
        for &parent in parents_arc.iter() {
            let mut children_map = self.children.write();
            let mut current = children_map
                .get(&parent)
                .map(|v| (**v).clone())
                .unwrap_or_default();
            current.insert(hash);
            children_map.insert(parent, Arc::new(current));
        }
        self.parents.write().insert(hash, parents_arc);
    }

    pub fn delete(&self, hash: Hash) {
        if let Some(parents) = self.parents.write().remove(&hash) {
            let mut children_map = self.children.write();
            for parent in parents.iter() {
                if let Some(entry) = children_map.get_mut(parent) {
                    let mut current = (**entry).clone();
                    current.remove(&hash);
                    *entry = Arc::new(current);
                }
            }
        }
        self.children.write().remove(&hash);
    }

    pub fn get_parents(&self, hash: &Hash) -> Option<Vec<BlockHash>> {
        <Self as RelationsStoreReader>::get_parents(self, *hash)
            .ok()
            .map(|v| (*v).clone())
    }

    pub fn get_children(&self, hash: &Hash) -> Option<Vec<BlockHash>> {
        <Self as RelationsStoreReader>::get_children(self, *hash)
            .ok()
            .map(|v| v.iter().copied().collect())
    }

    pub fn has(&self, hash: &Hash) -> bool {
        <Self as RelationsStoreReader>::has(self, *hash).unwrap_or(false)
    }
}

impl RelationsStoreReader for RelationsStore {
    fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        self.parents
            .read()
            .get(&hash)
            .cloned()
            .ok_or_else(|| StoreError::KeyNotFound(format!("relations parents for {hash} not found")))
    }

    fn get_children(&self, hash: Hash) -> StoreResult<ReadLock<BlockHashSet>> {
        self.children
            .read()
            .get(&hash)
            .cloned()
            .ok_or_else(|| StoreError::KeyNotFound(format!("relations children for {hash} not found")))
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        Ok(self.parents.read().contains_key(&hash))
    }

    fn counts(&self) -> Result<(usize, usize), StoreError> {
        Ok((self.parents.read().len(), self.children.read().len()))
    }
}
