use jio_consensus_core::blockhash::BlockHash;
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait RelationsStoreReader {
    fn get_parents(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>>;
    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>>;
    fn has(&self, hash: Hash) -> bool;
}

#[derive(Default, Clone)]
pub struct RelationsStore {
    parents: Arc<RwLock<HashMap<Hash, Arc<Vec<BlockHash>>>>>,
    children: Arc<RwLock<HashMap<Hash, Arc<Vec<BlockHash>>>>>,
}

impl RelationsStore {
    pub fn new() -> Self {
        Self {
            parents: Arc::new(RwLock::new(HashMap::new())),
            children: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, hash: Hash, parents: Vec<BlockHash>) {
        for parent in &parents {
            let mut children_map = self.children.write();
            let mut current = children_map.get(parent).map(|v| (**v).clone()).unwrap_or_default();
            current.push(hash);
            children_map.insert(*parent, Arc::new(current));
        }
        self.parents.write().insert(hash, Arc::new(parents));
    }

    pub fn delete(&self, hash: Hash) {
        self.parents.write().remove(&hash);
        self.children.write().remove(&hash);
    }
}

impl RelationsStoreReader for RelationsStore {
    fn get_parents(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>> {
        self.parents.read().get(&hash).cloned()
    }

    fn get_children(&self, hash: Hash) -> Option<Arc<Vec<BlockHash>>> {
        self.children.read().get(&hash).cloned()
    }

    fn has(&self, hash: Hash) -> bool {
        self.parents.read().contains_key(&hash)
    }
}

// Ergonomic helper methods accepting both &Hash and Hash
impl RelationsStore {
    pub fn get_parents(&self, hash: &Hash) -> Option<Vec<BlockHash>> {
        <Self as RelationsStoreReader>::get_parents(self, *hash).map(|v| (*v).clone())
    }

    pub fn get_children(&self, hash: &Hash) -> Option<Vec<BlockHash>> {
        <Self as RelationsStoreReader>::get_children(self, *hash).map(|v| (*v).clone())
    }

    pub fn has(&self, hash: &Hash) -> bool {
        <Self as RelationsStoreReader>::has(self, *hash)
    }
}
