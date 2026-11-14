use crate::model::stores::reachability::{Interval, ReachabilityData, ReachabilityStore};
use crate::model::stores::relations::RelationsStoreReader;
use jio_database::prelude::{DbWriter, StoreResult};
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::sync::Arc;

pub const REINDEX_THRESHOLD: u64 = 8;

pub trait ReachabilityService: Send + Sync {
    fn is_dag_ancestor_of(&self, this: Hash, queried: Hash) -> bool;
    fn is_chain_ancestor_of(&self, this: Hash, queried: Hash) -> bool;
}

pub struct ReachabilityManager<TStore: ReachabilityStore, TRelations: RelationsStoreReader> {
    store: Arc<RwLock<TStore>>,
    relations: Arc<TRelations>,
}

impl<TStore: ReachabilityStore, TRelations: RelationsStoreReader>
    ReachabilityManager<TStore, TRelations>
{
    pub fn new(store: Arc<RwLock<TStore>>, relations: Arc<TRelations>) -> Self {
        Self { store, relations }
    }

    /// Initializes reachability store with the genesis block occupying the maximal interval.
    pub fn init(&self, writer: &mut impl DbWriter, genesis: Hash) -> StoreResult<()> {
        let store = self.store.write();
        if !store.has(genesis) {
            let data = Arc::new(ReachabilityData::new(
                Interval::maximal(),
                Hash::default(),
                Vec::new(),
            ));
            store.insert(writer, genesis, data)?;
        }
        Ok(())
    }

    /// Adds a new block to the reachability tree under its selected parent.
    pub fn add_block(
        &self,
        writer: &mut impl DbWriter,
        block: Hash,
        selected_parent: Hash,
    ) -> StoreResult<()> {
        let store = self.store.write();
        let parent_interval = store.get_interval(selected_parent)?;
        let parent_children = store.get_children(selected_parent)?;

        // Allocate a sub-interval inside parent interval
        let num_children = parent_children.len() as u64;
        let parent_size = parent_interval.size();
        let chunk_size = (parent_size / (num_children + 2)).max(1);

        let child_start = parent_interval.start + (num_children + 1) * chunk_size;
        let child_end = (child_start + chunk_size).min(parent_interval.end);
        let child_interval = Interval::new(child_start, child_end);

        let data = Arc::new(ReachabilityData::new(
            child_interval,
            selected_parent,
            Vec::new(),
        ));
        store.insert(writer, block, data)?;
        store.append_child(writer, selected_parent, block)?;
        Ok(())
    }
}

impl<TStore: ReachabilityStore + Send + Sync, TRelations: RelationsStoreReader + Send + Sync>
    ReachabilityService for ReachabilityManager<TStore, TRelations>
{
    /// O(1) ancestry check via interval containment along the spanning tree,
    /// with fallback to BFS over DAG relations for non-tree edges.
    fn is_dag_ancestor_of(&self, this: Hash, queried: Hash) -> bool {
        if this == queried {
            return true;
        }
        let store = self.store.read();
        let (Ok(this_interval), Ok(queried_interval)) =
            (store.get_interval(this), store.get_interval(queried))
        else {
            return false;
        };

        // Tree-based fast path: O(1) check
        if this_interval.contains(&queried_interval) {
            return true;
        }

        // Drop store lock before DAG traversal
        drop(store);

        // DAG-path check (for non-tree edges): Breadth-First Search on parent relations
        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        queue.push_back(queried);
        visited.insert(queried);

        while let Some(current) = queue.pop_front() {
            if current == this {
                return true;
            }
            if let Ok(parents) = self.relations.get_parents(current) {
                for &parent in parents.iter() {
                    if parent == this {
                        return true;
                    }
                    if visited.insert(parent) {
                        queue.push_back(parent);
                    }
                }
            }
        }

        false
    }

    fn is_chain_ancestor_of(&self, this: Hash, queried: Hash) -> bool {
        let store = self.store.read();
        let (Ok(this_interval), Ok(queried_interval)) =
            (store.get_interval(this), store.get_interval(queried))
        else {
            return false;
        };
        this_interval.contains(&queried_interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::reachability::DbReachabilityStore;
    use crate::model::stores::relations::DbRelationsStore;
    use jio_database::prelude::{DirectDbWriter, open_db};
    use tempfile::tempdir;

    #[test]
    fn test_reachability_ancestry() {
        let temp = tempdir().unwrap();
        let db = open_db(temp.path().to_path_buf(), true, 1);
        let store = Arc::new(RwLock::new(DbReachabilityStore::new(db.clone(), 100)));
        let relations = Arc::new(DbRelationsStore::new(db.clone(), 100));
        let manager = ReachabilityManager::new(store, relations);
        let mut writer = DirectDbWriter::new(&db);

        let genesis = Hash::from_bytes([1u8; 32]);
        let block_a = Hash::from_bytes([2u8; 32]);
        let block_b = Hash::from_bytes([3u8; 32]);

        manager.init(&mut writer, genesis).unwrap();
        manager.add_block(&mut writer, block_a, genesis).unwrap();
        manager.add_block(&mut writer, block_b, block_a).unwrap();

        assert!(manager.is_dag_ancestor_of(genesis, block_a));
        assert!(manager.is_dag_ancestor_of(genesis, block_b));
        assert!(manager.is_dag_ancestor_of(block_a, block_b));
        assert!(!manager.is_dag_ancestor_of(block_b, block_a));
    }
}
