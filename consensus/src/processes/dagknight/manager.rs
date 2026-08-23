use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, OnceLock},
};

use parking_lot::{Mutex, RwLock};

use jio_consensus_core::{
    blockhash::BlockHashExtensions,
    BlockHashMap, BlockHashSet, BlueWorkType, HashMapCustomHasher, HashKTypeMap, KType,
};
use jio_database::prelude::StoreError;
use jio_hashes::Hash;

use crate::{
    model::{
        services::reachability::{MTReachabilityService, ReachabilityService},
        stores::{
            dagknight::{DagknightKey, DagknightStore, DagknightStoreReader},
            ghostdag::GhostdagData,
            headers::HeaderStoreReader,
            reachability::ReachabilityStoreReader,
            relations::RelationsStoreReader,
        },
    },
    processes::{
        difficulty::calc_work,
        ghostdag::ordering::SortableBlock,
        reachability::relations::FutureIntersectRelations,
    },
};

struct ChainBlock {
    hash: Option<Hash>,
    data: GhostdagData,
}

enum ColoringState {
    Blue,
    Red,
    Pending,
}

pub(crate) enum ColoringOutput {
    Blue(KType, BlockHashMap<KType>),
    Red,
}

/// Granular lock key for k-colouring operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KColouringLockKey {
    conflict_genesis: Hash,
    k: KType,
    nca: Hash,
    free_search: bool,
}

impl KColouringLockKey {
    pub fn committed_search_key(conflict_genesis: Hash, k: KType, nca: Hash) -> Self {
        Self { conflict_genesis, k, nca, free_search: false }
    }

    pub fn free_search_key(conflict_genesis: Hash, k: KType) -> Self {
        Self { conflict_genesis, k, nca: conflict_genesis, free_search: true }
    }

    pub fn conflict_genesis(&self) -> Hash {
        self.conflict_genesis
    }

    pub fn is_free_search(&self) -> bool {
        self.free_search
    }
}

static K_COLOURING_LOCKS: OnceLock<Mutex<HashMap<KColouringLockKey, Arc<RwLock<()>>>>> = OnceLock::new();

fn get_k_colouring_lock(key: &KColouringLockKey) -> Arc<RwLock<()>> {
    let map_mutex = K_COLOURING_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = map_mutex.lock();
    map.entry(key.clone()).or_insert_with(|| Arc::new(RwLock::new(()))).clone()
}

pub struct ConflictZoneManager<
    C: DagknightStore + DagknightStoreReader,
    O: HeaderStoreReader,
    D: RelationsStoreReader,
    R: ReachabilityStoreReader + Clone,
> {
    k: KType,
    root: Hash,
    free_search: bool,
    dagknight_store: Arc<C>,
    headers_store: Arc<O>,
    relations_store: FutureIntersectRelations<D, MTReachabilityService<R>>,
    reachability_service: MTReachabilityService<R>,
}

impl<C: DagknightStore + DagknightStoreReader, O: HeaderStoreReader, D: RelationsStoreReader, R: ReachabilityStoreReader + Clone>
    ConflictZoneManager<C, O, D, R>
{
    pub fn new(
        k: KType,
        root: Hash,
        dagknight_store: Arc<C>,
        headers_store: Arc<O>,
        relations_store: FutureIntersectRelations<D, MTReachabilityService<R>>,
        reachability_service: MTReachabilityService<R>,
    ) -> Self {
        Self { k, root, free_search: false, dagknight_store, headers_store, reachability_service, relations_store }
    }

    pub fn with_free_search(
        k: KType,
        root: Hash,
        dagknight_store: Arc<C>,
        headers_store: Arc<O>,
        relations_store: FutureIntersectRelations<D, MTReachabilityService<R>>,
        reachability_service: MTReachabilityService<R>,
        free_search: bool,
    ) -> Self {
        Self { k, root, free_search, dagknight_store, headers_store, reachability_service, relations_store }
    }

    pub fn has(&self, pov_hash: Hash) -> bool {
        let key = self.get_key(pov_hash);
        self.dagknight_store.has(key).unwrap_or(false)
    }

    pub fn insert(&self, pov_hash: Hash, gd: Arc<GhostdagData>) -> Result<(), StoreError> {
        let key = self.get_key(pov_hash);
        self.dagknight_store.insert(key, gd)
    }

    fn get_key(&self, pov_hash: Hash) -> DagknightKey {
        DagknightKey::new(self.root, pov_hash, self.k, self.free_search)
    }

    pub fn get_blue_score(&self, pov_hash: Hash) -> Result<u64, StoreError> {
        let key = self.get_key(pov_hash);
        Ok(self.dagknight_store.get_data(key)?.blue_score)
    }

    pub fn get_blue_work(&self, pov_hash: Hash) -> Result<BlueWorkType, StoreError> {
        let key = self.get_key(pov_hash);
        Ok(self.dagknight_store.get_data(key)?.blue_work)
    }

    pub fn get_selected_parent(&self, pov_hash: Hash) -> Result<Hash, StoreError> {
        let key = self.get_key(pov_hash);
        Ok(self.dagknight_store.get_data(key)?.selected_parent)
    }

    pub fn get_blues_anticone_sizes(&self, pov_hash: Hash) -> Result<HashKTypeMap, StoreError> {
        let key = self.get_key(pov_hash);
        Ok(self.dagknight_store.get_data(key)?.blues_anticone_sizes.clone())
    }

    pub fn get_data(&self, pov_hash: Hash) -> Result<Arc<GhostdagData>, StoreError> {
        let key = self.get_key(pov_hash);
        self.dagknight_store.get_data(key)
    }

    pub fn find_selected_parent(&self, parents: impl IntoIterator<Item = Hash>) -> Hash {
        parents
            .into_iter()
            .map(|parent| {
                let blue_work = if parent == self.root {
                    BlueWorkType::from(0u64)
                } else {
                    self.get_blue_work(parent).unwrap_or_else(|_| BlueWorkType::from(0u64))
                };
                SortableBlock { hash: parent, blue_work }
            })
            .max()
            .unwrap()
            .hash
    }

    pub fn ordered_mergeset_without_selected_parent(&self, selected_parent: Hash, parents: &[Hash]) -> Vec<Hash> {
        let mut queue: VecDeque<Hash> = parents.iter().copied().filter(|&p| p != selected_parent).collect();
        let mut visited: BlockHashSet = queue.iter().copied().collect();
        let mut mergeset = Vec::new();

        while let Some(current) = queue.pop_front() {
            if current == self.root || self.reachability_service.is_dag_ancestor_of(current, selected_parent) {
                continue;
            }
            mergeset.push(current);

            if let Ok(current_parents) = self.relations_store.get_parents(current) {
                for &parent in current_parents.iter() {
                    if visited.insert(parent) {
                        queue.push_back(parent);
                    }
                }
            }
        }

        // Sort topologically forward in time
        mergeset.sort_by_cached_key(|&h| {
            if let Ok(data) = self.get_data(h) {
                data.blue_score
            } else {
                0
            }
        });

        mergeset
    }

    pub fn k_colouring(&self, parents: &[Hash], k: KType, custom_selected_parent: Option<Hash>) -> GhostdagData {
        assert!(!parents.is_empty(), "parents cannot be empty in k_colouring");

        let selected_parent = custom_selected_parent.unwrap_or_else(|| self.find_selected_parent(parents.iter().copied()));
        if selected_parent.is_origin() || selected_parent == self.root {
            return GhostdagData::new_with_selected_parent(selected_parent, k);
        }

        let mut new_block_data = GhostdagData::new_with_selected_parent(selected_parent, k);
        let ordered_mergeset = self.ordered_mergeset_without_selected_parent(selected_parent, parents);

        for blue_candidate in ordered_mergeset {
            let coloring = self.check_blue_candidate(&new_block_data, blue_candidate, k);
            if let ColoringOutput::Blue(blue_anticone_size, blues_anticone_sizes) = coloring {
                new_block_data.add_blue(blue_candidate, blue_anticone_size, &blues_anticone_sizes);
            } else {
                new_block_data.add_red(blue_candidate);
            }
        }

        let base_blue_score = self.get_blue_score(selected_parent).unwrap_or(0);
        let blue_score = base_blue_score + new_block_data.mergeset_blues.len() as u64;

        let added_blue_work: BlueWorkType = new_block_data
            .mergeset_blues
            .iter()
            .cloned()
            .map(|hash| {
                if hash == self.root || hash.is_origin() {
                    BlueWorkType::from(0u64)
                } else {
                    calc_work(self.headers_store.get_bits(hash).unwrap_or(0))
                }
            })
            .sum();

        let base_blue_work = self.get_blue_work(selected_parent).unwrap_or_else(|_| BlueWorkType::from(0u64));
        let blue_work = base_blue_work + added_blue_work;
        new_block_data.finalize_score_and_work(blue_score, blue_work);

        new_block_data
    }

    fn check_blue_candidate_with_chain_block(
        &self,
        _new_block_data: &GhostdagData,
        chain_block: &ChainBlock,
        blue_candidate: Hash,
        candidate_blues_anticone_sizes: &mut BlockHashMap<KType>,
        candidate_blue_anticone_size: &mut KType,
        k: KType,
    ) -> ColoringState {
        if let Some(hash) = chain_block.hash {
            if self.reachability_service.is_dag_ancestor_of(hash, blue_candidate) {
                return ColoringState::Blue;
            }
        }

        for &blue in chain_block.data.mergeset_blues.iter() {
            if blue == self.root || blue == chain_block.data.selected_parent {
                continue;
            }

            if !self.reachability_service.is_dag_ancestor_of(blue, blue_candidate) {
                *candidate_blue_anticone_size += 1;
                if *candidate_blue_anticone_size > k {
                    return ColoringState::Red;
                }

                let current_blue_anticone_size = candidate_blues_anticone_sizes
                    .entry(blue)
                    .or_insert_with(|| chain_block.data.blues_anticone_sizes.get(&blue).copied().unwrap_or(0));

                *current_blue_anticone_size += 1;
                if *current_blue_anticone_size > k {
                    return ColoringState::Red;
                }
            }
        }

        ColoringState::Pending
    }

    pub fn check_blue_candidate(&self, new_block_data: &GhostdagData, blue_candidate: Hash, k: KType) -> ColoringOutput {
        let mut candidate_blues_anticone_sizes: BlockHashMap<KType> = BlockHashMap::new();
        let mut candidate_blue_anticone_size: KType = 0;

        let mut current_chain_block = ChainBlock { hash: None, data: new_block_data.clone() };

        loop {
            let state = self.check_blue_candidate_with_chain_block(
                new_block_data,
                &current_chain_block,
                blue_candidate,
                &mut candidate_blues_anticone_sizes,
                &mut candidate_blue_anticone_size,
                k,
            );

            match state {
                ColoringState::Blue => return ColoringOutput::Blue(candidate_blue_anticone_size, candidate_blues_anticone_sizes),
                ColoringState::Red => return ColoringOutput::Red,
                ColoringState::Pending => {}
            }

            let sp = current_chain_block.data.selected_parent;
            if sp == self.root || sp.is_origin() {
                break;
            }

            if let Ok(sp_data) = self.get_data(sp) {
                current_chain_block = ChainBlock { hash: Some(sp), data: (*sp_data).clone() };
            } else {
                break;
            }
        }

        ColoringOutput::Blue(candidate_blue_anticone_size, candidate_blues_anticone_sizes)
    }

    pub fn fill_zone_data(&self, tips: &[Hash], nca: Option<Hash>) {
        let lock_key = if let Some(nca_hash) = nca {
            KColouringLockKey::committed_search_key(self.root, self.k, nca_hash)
        } else {
            KColouringLockKey::free_search_key(self.root, self.k)
        };
        let _guard = get_k_colouring_lock(&lock_key);
        let _write_lock = _guard.write();

        let mut queue: VecDeque<Hash> = tips.iter().copied().collect();
        let mut visited: BlockHashSet = queue.iter().copied().collect();
        let mut topo_order: Vec<Hash> = Vec::new();

        while let Some(current) = queue.pop_front() {
            if current == self.root {
                continue;
            }
            topo_order.push(current);

            if let Ok(parents) = self.relations_store.get_parents(current) {
                for &parent in parents.iter() {
                    if parent != self.root && visited.insert(parent) {
                        queue.push_back(parent);
                    }
                }
            }
        }

        topo_order.reverse();

        for &block in &topo_order {
            if self.has(block) {
                continue;
            }

            if let Ok(parents) = self.relations_store.get_parents(block) {
                let gd = self.k_colouring(&parents, self.k, None);
                let _ = self.insert(block, Arc::new(gd));
            }
        }
    }
}
