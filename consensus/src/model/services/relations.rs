use crate::model::stores::relations::RelationsStoreReader;
use jio_consensus_core::blockhash::BlockHashes;
use jio_consensus_core::BlockHashSet;
use jio_database::prelude::{ReadLock, StoreError, StoreResult};
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::sync::Arc;

/// Multi-threaded block-relations service imp
#[derive(Clone)]
pub struct MTRelationsService<T: RelationsStoreReader> {
    store: Arc<RwLock<Vec<T>>>,
    level: usize,
}

impl<T: RelationsStoreReader> MTRelationsService<T> {
    pub fn new(store: Arc<RwLock<Vec<T>>>, level: u8) -> Self {
        Self {
            store,
            level: level as usize,
        }
    }
}

impl<T: RelationsStoreReader> RelationsStoreReader for MTRelationsService<T> {
    fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        self.store.read()[self.level].get_parents(hash)
    }

    fn get_children(&self, hash: Hash) -> StoreResult<ReadLock<BlockHashSet>> {
        self.store.read()[self.level].get_children(hash)
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        self.store.read()[self.level].has(hash)
    }

    fn counts(&self) -> Result<(usize, usize), StoreError> {
        self.store.read()[self.level].counts()
    }
}

#[derive(Clone)]
pub struct RelationsService<T: RelationsStoreReader> {
    store: T,
}

impl<T: RelationsStoreReader> RelationsService<T> {
    pub fn new(store: T) -> Self {
        Self { store }
    }

    pub fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        self.store.get_parents(hash)
    }

    pub fn get_children(&self, hash: Hash) -> StoreResult<ReadLock<BlockHashSet>> {
        self.store.get_children(hash)
    }

    pub fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        self.store.has(hash)
    }

    pub fn counts(&self) -> Result<(usize, usize), StoreError> {
        self.store.counts()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::relations::MemoryRelationsStore;

    #[test]
    fn test_mt_relations_service() {
        let mut store_level0 = MemoryRelationsStore::new();
        let mut store_level1 = MemoryRelationsStore::new();

        let parent: Hash = 1.into();
        let child1: Hash = 2.into();
        let child2: Hash = 3.into();

        store_level0.insert(child1, Arc::new(vec![parent]));
        store_level0.insert(child2, Arc::new(vec![parent]));

        store_level1.insert(child1, Arc::new(vec![parent]));

        let stores = Arc::new(RwLock::new(vec![store_level0, store_level1]));
        let service_l0 = MTRelationsService::new(stores.clone(), 0);
        let service_l1 = MTRelationsService::new(stores.clone(), 1);

        assert!(service_l0.has(child1).unwrap());
        assert!(service_l0.has(child2).unwrap());
        assert_eq!(service_l0.get_parents(child1).unwrap().as_slice(), &[parent]);

        let children_l0 = service_l0.get_children(parent).unwrap();
        assert_eq!(children_l0.len(), 2);
        assert!(children_l0.contains(&child1));
        assert!(children_l0.contains(&child2));

        let (parent_count, child_count) = service_l0.counts().unwrap();
        assert_eq!(parent_count, 2);
        assert_eq!(child_count, 1);

        assert!(service_l1.has(child1).unwrap());
        assert!(!service_l1.has(child2).unwrap());
    }
}
