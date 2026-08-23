use std::sync::Arc;

use parking_lot::RwLock;

use jio_consensus_core::{blockhash::BlockHashExtensions, BlockHashSet, KType};
use jio_hashes::Hash;

use crate::{
    model::{
        services::reachability::{MTReachabilityService, ReachabilityService},
        stores::{
            dagknight::{DagknightStore, DagknightStoreReader},
            headers::HeaderStoreReader,
            reachability::ReachabilityStoreReader,
            relations::RelationsStoreReader,
        },
    },
    processes::{
        dagknight::{manager::ConflictZoneManager, GroupMetadata},
        reachability::relations::FutureIntersectRelations,
    },
};

/// Chain blocks from a subgroup's conditioned k-colouring.
pub type SubgroupChainBlocks = Vec<Hash>;

/// Result of free-search k-colouring reference cluster computation.
pub struct ReferenceCluster {
    /// Set of blue block hashes in the resulting colouring.
    pub blues: BlockHashSet,
    /// Chain backbone from virtual towards conflict_genesis (inclusive).
    pub chain_blocks: Vec<Hash>,
}

/// Input data for a tie-breaking call.
pub struct TieBreakContext<'a> {
    pub conflict_genesis: Hash,
    pub all_tips: &'a [Hash],
    pub subgroups: &'a [GroupMetadata],
    pub k: KType,
}

/// Trait for tie-breaking logic.
pub trait TieBreaker {
    fn tie_break(&self, input: &TieBreakContext<'_>) -> usize;
}

/// DAGKnight tie-breaker backed by the consensus stores.
pub struct DagknightTieBreaker<
    C: DagknightStore + DagknightStoreReader,
    O: HeaderStoreReader + 'static,
    D: RelationsStoreReader + Clone,
    R: ReachabilityStoreReader + Clone,
> {
    pub dagknight_store: Arc<C>,
    pub headers_store: Arc<O>,
    pub relations_store: Arc<RwLock<D>>,
    pub reachability_service: MTReachabilityService<R>,
}

impl<
    C: DagknightStore + DagknightStoreReader,
    O: HeaderStoreReader + 'static,
    D: RelationsStoreReader + Clone,
    R: ReachabilityStoreReader + Clone,
> DagknightTieBreaker<C, O, D, R>
{
    pub fn new(
        dagknight_store: Arc<C>,
        headers_store: Arc<O>,
        relations_store: Arc<RwLock<D>>,
        reachability_service: MTReachabilityService<R>,
    ) -> Self {
        Self { dagknight_store, headers_store, relations_store, reachability_service }
    }

    pub fn compute_reference_cluster(&self, conflict_genesis: Hash, all_tips: &[Hash], k: KType) -> ReferenceCluster {
        let reachability_service = self.reachability_service.clone();
        let relations_store = self.relations_store.read().clone();
        let relations_service = FutureIntersectRelations::new(relations_store, reachability_service.clone(), conflict_genesis);

        let conflict_zone_manager = ConflictZoneManager::with_free_search(
            k,
            conflict_genesis,
            self.dagknight_store.clone(),
            self.headers_store.clone(),
            relations_service,
            reachability_service.clone(),
            true,
        );

        conflict_zone_manager.fill_zone_data(all_tips, None);

        let virtual_gd = conflict_zone_manager.k_colouring(all_tips, k, None);

        let mut blue_set: BlockHashSet = BlockHashSet::default();
        let mut chain_blocks: Vec<Hash> = Vec::new();

        for &blue_block in virtual_gd.mergeset_blues.iter() {
            blue_set.insert(blue_block);
        }

        let mut curr_sp = virtual_gd.selected_parent;
        while curr_sp != conflict_genesis && !curr_sp.is_origin() {
            chain_blocks.push(curr_sp);
            blue_set.insert(curr_sp);
            if let Ok(gd) = conflict_zone_manager.get_data(curr_sp) {
                for &blue_block in gd.mergeset_blues.iter() {
                    blue_set.insert(blue_block);
                }
                curr_sp = gd.selected_parent;
            } else {
                break;
            }
        }
        blue_set.insert(conflict_genesis);
        chain_blocks.push(conflict_genesis);

        ReferenceCluster { blues: blue_set, chain_blocks }
    }

    pub fn compute_subgroup_chain_blocks(
        &self,
        conflict_genesis: Hash,
        group_tips: &[Hash],
        all_tips: &[Hash],
        k_prime: KType,
    ) -> SubgroupChainBlocks {
        let reachability_service = self.reachability_service.clone();
        let relations_store = self.relations_store.read().clone();
        let relations_service = FutureIntersectRelations::new(relations_store, reachability_service.clone(), conflict_genesis);

        let conflict_zone_manager = ConflictZoneManager::with_free_search(
            k_prime,
            conflict_genesis,
            self.dagknight_store.clone(),
            self.headers_store.clone(),
            relations_service,
            reachability_service.clone(),
            false,
        );

        let group_selected_parent = conflict_zone_manager.find_selected_parent(group_tips.iter().copied());
        let nca = self.reachability_service.get_next_chain_ancestor(group_selected_parent, conflict_genesis);
        conflict_zone_manager.fill_zone_data(all_tips, Some(nca));

        let virtual_gd = conflict_zone_manager.k_colouring(all_tips, k_prime, Some(group_selected_parent));

        let mut chain_blocks: Vec<Hash> = Vec::new();
        let mut curr_sp = virtual_gd.selected_parent;
        while curr_sp != conflict_genesis && !curr_sp.is_origin() {
            chain_blocks.push(curr_sp);
            if let Ok(gd) = conflict_zone_manager.get_data(curr_sp) {
                curr_sp = gd.selected_parent;
            } else {
                break;
            }
        }
        chain_blocks.push(conflict_genesis);

        chain_blocks
    }
}

impl<
    C: DagknightStore + DagknightStoreReader,
    O: HeaderStoreReader + 'static,
    D: RelationsStoreReader + Clone,
    R: ReachabilityStoreReader + Clone,
> TieBreaker for DagknightTieBreaker<C, O, D, R>
{
    fn tie_break(&self, input: &TieBreakContext<'_>) -> usize {
        assert!(!input.subgroups.is_empty(), "TieBreakContext must contain at least one subgroup");
        if input.subgroups.len() == 1 {
            return 0;
        }

        let ref_cluster = self.compute_reference_cluster(input.conflict_genesis, input.all_tips, input.k);

        let mut best_index = 0;
        let mut best_overlap: usize = 0;
        let mut best_selected_parent = input.subgroups[0].selected_parent.clone();

        for (idx, group) in input.subgroups.iter().enumerate() {
            let chain_blocks = self.compute_subgroup_chain_blocks(input.conflict_genesis, &group.subgroup, input.all_tips, input.k);
            let overlap = chain_blocks.iter().filter(|b| ref_cluster.blues.contains(b)).count();

            if idx == 0 {
                best_overlap = overlap;
                best_selected_parent = group.selected_parent.clone();
                best_index = 0;
                continue;
            }

            if overlap > best_overlap {
                best_overlap = overlap;
                best_selected_parent = group.selected_parent.clone();
                best_index = idx;
            } else if overlap == best_overlap {
                if group.selected_parent > best_selected_parent {
                    best_selected_parent = group.selected_parent.clone();
                    best_index = idx;
                }
            }
        }

        best_index
    }
}
