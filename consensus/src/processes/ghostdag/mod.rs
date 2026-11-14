pub mod mergeset;
pub mod ordering;
pub mod protocol;

use crate::model::stores::ghostdag::{GhostdagData, GhostdagStore};
use crate::model::stores::headers::HeaderStoreReader;
use crate::model::stores::relations::RelationsStoreReader;
use crate::processes::reachability::ReachabilityService;
use consensus_core::{BlockHashMap, BlueWorkType, HashMapCustomHasher, KType, calc_work};
use jio_database::prelude::{DbWriter, StoreResult};
use jio_hashes::Hash;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct GhostdagManager<
    TStore: GhostdagStore,
    TRelations: RelationsStoreReader,
    THeaders: HeaderStoreReader,
    TReachability: ReachabilityService,
> {
    k: KType,
    store: Arc<RwLock<TStore>>,
    relations: Arc<TRelations>,
    headers: Arc<THeaders>,
    reachability: Arc<TReachability>,
}

impl<
    TStore: GhostdagStore,
    TRelations: RelationsStoreReader,
    THeaders: HeaderStoreReader,
    TReachability: ReachabilityService,
> GhostdagManager<TStore, TRelations, THeaders, TReachability>
{
    pub fn new(
        k: KType,
        store: Arc<RwLock<TStore>>,
        relations: Arc<TRelations>,
        headers: Arc<THeaders>,
        reachability: Arc<TReachability>,
    ) -> Self {
        Self {
            k,
            store,
            relations,
            headers,
            reachability,
        }
    }

    /// Initializes GHOSTDAG consensus data for the genesis block.
    pub fn init_genesis(
        &self,
        writer: &mut impl DbWriter,
        genesis: Hash,
    ) -> StoreResult<Arc<GhostdagData>> {
        let store = self.store.write();
        if let Ok(existing) = store.get(genesis) {
            return Ok(existing);
        }

        let genesis_data = Arc::new(GhostdagData::new(
            0,
            BlueWorkType::ZERO,
            Hash::default(),
            Vec::new(),
            Vec::new(),
            BlockHashMap::new(),
        ));

        store.insert(writer, genesis, genesis_data.clone())?;
        Ok(genesis_data)
    }

    /// Computes GHOSTDAG data for a newly added block and stores it in the store.
    pub fn add_block(
        &self,
        writer: &mut impl DbWriter,
        block: Hash,
        parents: Arc<Vec<Hash>>,
    ) -> StoreResult<Arc<GhostdagData>> {
        let store = self.store.read();

        // If block has no parents, it must be genesis
        if parents.is_empty() {
            drop(store);
            return self.init_genesis(writer, block);
        }

        // 1. Find selected parent
        let selected_parent =
            protocol::find_selected_parent(&*store, &parents).expect("parents list is non-empty");

        let selected_parent_data = store.get(selected_parent)?;

        // 2. Calculate merge set
        let mut mergeset = mergeset::calculate_mergeset(
            &*self.relations,
            &*self.reachability,
            &parents,
            selected_parent,
        );

        // 3. Sort merge set topologically
        ordering::sort_mergeset(&*store, &mut mergeset);

        // 4. Color merge set into blues and reds according to K-cluster rule
        let (mergeset_blues, mergeset_reds, blues_anticone_sizes) = protocol::evaluate_mergeset(
            &*store,
            &*self.reachability,
            self.k,
            selected_parent,
            &mergeset,
        );

        // 5. Accumulate blue score and blue work
        let blue_score = selected_parent_data.blue_score + 1 + mergeset_blues.len() as u64;

        // Selected parent is blue; accumulate its work
        let selected_parent_bits = self.headers.get_bits(selected_parent).unwrap_or(0);
        let mut blue_work = selected_parent_data.blue_work + calc_work(selected_parent_bits);

        // Accumulate work of all mergeset blues
        for &blue in &mergeset_blues {
            let blue_bits = self.headers.get_bits(blue).unwrap_or(0);
            blue_work = blue_work + calc_work(blue_bits);
        }

        let ghostdag_data = Arc::new(GhostdagData::new(
            blue_score,
            blue_work,
            selected_parent,
            mergeset_blues,
            mergeset_reds,
            blues_anticone_sizes,
        ));

        drop(store);
        let store = self.store.write();
        store.insert(writer, block, ghostdag_data.clone())?;

        Ok(ghostdag_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stores::ghostdag::DbGhostdagStore;
    use crate::model::stores::headers::DbHeaderStore;
    use crate::model::stores::headers::HeaderStore;
    use crate::model::stores::reachability::DbReachabilityStore;
    use crate::model::stores::relations::DbRelationsStore;
    use crate::model::stores::relations::RelationsStore;
    use crate::processes::reachability::ReachabilityManager;
    use consensus_core::header::Header;
    use jio_database::prelude::{BatchDbWriter, DB};
    use tempfile::TempDir;

    #[test]
    fn test_ghostdag_linear_chain_and_branching() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(DB::open_default(temp_dir.path()).unwrap());
        let mut writer = BatchDbWriter::new(&db);

        let ghostdag_store = Arc::new(RwLock::new(DbGhostdagStore::new(db.clone(), 100)));
        let relations_store = Arc::new(DbRelationsStore::new(db.clone(), 100));
        let headers_store = Arc::new(DbHeaderStore::new(db.clone(), 100));
        let reachability_store = Arc::new(RwLock::new(DbReachabilityStore::new(db.clone(), 100)));

        let reachability_manager = Arc::new(ReachabilityManager::new(
            reachability_store.clone(),
            relations_store.clone(),
        ));

        let k: KType = 3;
        let ghostdag_manager = GhostdagManager::new(
            k,
            ghostdag_store.clone(),
            relations_store.clone(),
            headers_store.clone(),
            reachability_manager.clone(),
        );

        // 1. Genesis block (Hash 1)
        let genesis_hash = Hash::from_le_u64([1, 0, 0, 0]);
        reachability_manager
            .init(&mut writer, genesis_hash)
            .unwrap();
        relations_store
            .insert(&mut writer, genesis_hash, Arc::new(vec![]))
            .unwrap();
        let genesis_header = Arc::new(Header::new_finalized(
            1,
            vec![],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            1000,
            0x1e7fffff,
            0,
            0,
            BlueWorkType::ZERO,
            0,
            Hash::default(),
        ));
        headers_store
            .insert(&mut writer, genesis_hash, genesis_header)
            .unwrap();
        let genesis_gd = ghostdag_manager
            .init_genesis(&mut writer, genesis_hash)
            .unwrap();
        assert_eq!(genesis_gd.blue_score, 0);
        assert_eq!(genesis_gd.selected_parent, Hash::default());

        // 2. Block A on top of Genesis (Hash 2)
        let block_a = Hash::from_le_u64([2, 0, 0, 0]);
        reachability_manager
            .add_block(&mut writer, block_a, genesis_hash)
            .unwrap();
        relations_store
            .insert(&mut writer, block_a, Arc::new(vec![genesis_hash]))
            .unwrap();
        relations_store
            .add_child(&mut writer, genesis_hash, block_a)
            .unwrap();
        let header_a = Arc::new(Header::new_finalized(
            1,
            vec![vec![genesis_hash]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            2000,
            0x1e7fffff,
            0,
            1,
            BlueWorkType::ZERO,
            1,
            Hash::default(),
        ));
        headers_store
            .insert(&mut writer, block_a, header_a)
            .unwrap();
        let gd_a = ghostdag_manager
            .add_block(&mut writer, block_a, Arc::new(vec![genesis_hash]))
            .unwrap();
        assert_eq!(gd_a.selected_parent, genesis_hash);
        assert_eq!(gd_a.blue_score, 1);
        assert_eq!(gd_a.mergeset_blues.len(), 0);

        // 3. Block B parallel to A on top of Genesis (Hash 3)
        let block_b = Hash::from_le_u64([3, 0, 0, 0]);
        reachability_manager
            .add_block(&mut writer, block_b, genesis_hash)
            .unwrap();
        relations_store
            .insert(&mut writer, block_b, Arc::new(vec![genesis_hash]))
            .unwrap();
        relations_store
            .add_child(&mut writer, genesis_hash, block_b)
            .unwrap();
        let header_b = Arc::new(Header::new_finalized(
            1,
            vec![vec![genesis_hash]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            2500,
            0x1e7fffff,
            0,
            1,
            BlueWorkType::ZERO,
            1,
            Hash::default(),
        ));
        headers_store
            .insert(&mut writer, block_b, header_b)
            .unwrap();
        let gd_b = ghostdag_manager
            .add_block(&mut writer, block_b, Arc::new(vec![genesis_hash]))
            .unwrap();
        assert_eq!(gd_b.selected_parent, genesis_hash);
        assert_eq!(gd_b.blue_score, 1);

        // 4. Block C merging A and B (Hash 4)
        // With parents [A, B], both have equal work, so tie-breaker picks smaller hash (block_a < block_b)
        let block_c = Hash::from_le_u64([4, 0, 0, 0]);
        reachability_manager
            .add_block(&mut writer, block_c, block_a)
            .unwrap();
        relations_store
            .insert(&mut writer, block_c, Arc::new(vec![block_a, block_b]))
            .unwrap();
        relations_store
            .add_child(&mut writer, block_a, block_c)
            .unwrap();
        relations_store
            .add_child(&mut writer, block_b, block_c)
            .unwrap();
        let header_c = Arc::new(Header::new_finalized(
            1,
            vec![vec![block_a, block_b]],
            Hash::default(),
            Hash::default(),
            Hash::default(),
            3000,
            0x1e7fffff,
            0,
            2,
            BlueWorkType::ZERO,
            2,
            Hash::default(),
        ));
        headers_store
            .insert(&mut writer, block_c, header_c)
            .unwrap();
        let gd_c = ghostdag_manager
            .add_block(&mut writer, block_c, Arc::new(vec![block_a, block_b]))
            .unwrap();
        assert_eq!(gd_c.selected_parent, block_a);
        // Block B is merged! Since K=3, B is well within anticone limit, so B is marked BLUE!
        assert_eq!(gd_c.mergeset_blues.len(), 1);
        assert_eq!(gd_c.mergeset_blues[0], block_b);
        assert_eq!(gd_c.mergeset_reds.len(), 0);
        // blue_score = A.blue_score(1) + 1 + mergeset_blues.len()(1) = 3
        assert_eq!(gd_c.blue_score, 3);
    }
}
