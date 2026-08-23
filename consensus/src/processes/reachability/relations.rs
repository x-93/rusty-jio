use itertools::Itertools;
use jio_consensus_core::{blockhash::BlockHashes, BlockHashSet};
use jio_database::prelude::{ReadLock, StoreError, StoreResult};
use jio_hashes::Hash;

use crate::model::{services::reachability::ReachabilityService, stores::relations::RelationsStoreReader};

/// A relations-store reader restricted to the future of a fixed root block (including the root).
///
/// Only parents and children that lie within the root’s future are exposed.
/// This provides a consistent, root-relative view of relations when operating on
/// proofs or subgraphs confined to that region of the DAG.
#[derive(Clone)]
pub struct FutureIntersectRelations<T: RelationsStoreReader, U: ReachabilityService> {
    relations_store: T,
    reachability_service: U,
    root: Hash,
}

impl<T: RelationsStoreReader, U: ReachabilityService> FutureIntersectRelations<T, U> {
    pub fn new(relations_store: T, reachability_service: U, root: Hash) -> Self {
        Self { relations_store, reachability_service, root }
    }
}

impl<T: RelationsStoreReader, U: ReachabilityService> RelationsStoreReader for FutureIntersectRelations<T, U> {
    fn get_parents(&self, hash: Hash) -> Result<BlockHashes, StoreError> {
        if hash == self.root {
            return Ok(BlockHashes::new(Vec::new()));
        }
        let parents = self.relations_store.get_parents(hash)?;
        let filtered_parents = parents
            .iter()
            .copied()
            .filter(|&parent| parent == self.root || self.reachability_service.is_dag_ancestor_of(self.root, parent))
            .collect_vec();
        Ok(BlockHashes::new(filtered_parents))
    }

    fn get_children(&self, hash: Hash) -> StoreResult<ReadLock<BlockHashSet>> {
        let children = self.relations_store.get_children(hash)?;
        let filtered_children: BlockHashSet = children
            .read()
            .iter()
            .copied()
            .filter(|&child| self.reachability_service.is_dag_ancestor_of(self.root, child))
            .collect();
        Ok(ReadLock::new(filtered_children))
    }

    fn has(&self, hash: Hash) -> Result<bool, StoreError> {
        if hash == self.root {
            return Ok(true);
        }
        if self.reachability_service.is_dag_ancestor_of(self.root, hash) {
            self.relations_store.has(hash)
        } else {
            Ok(false)
        }
    }

    fn counts(&self) -> Result<(usize, usize), StoreError> {
        self.relations_store.counts()
    }
}
