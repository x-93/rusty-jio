use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use itertools::Itertools;
use parking_lot::RwLock;

use jio_consensus_core::BlueWorkType;
use jio_core::debug;
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
        dagknight::{
            manager::ConflictZoneManager,
            rank_search::RankSearcher,
            tie_breaking::{DagknightTieBreaker, TieBreakContext, TieBreaker},
            umc_cascade::{BlockWithWork, CascadeMaintainer},
            DagknightCounters, GroupMetadata,
        },
        difficulty::calc_work,
        ghostdag::ordering::SortableBlock,
        reachability::relations::FutureIntersectRelations,
    },
};

#[derive(Clone, Debug)]
pub struct DagknightData {
    pub selected_parent: Hash,
    pub conflict_ordered_parents: Vec<Hash>,
}

#[derive(Clone)]
pub struct DagknightExecutor<
    C: DagknightStore + DagknightStoreReader,
    O: HeaderStoreReader + 'static,
    D: RelationsStoreReader + Clone,
    R: ReachabilityStoreReader + Clone,
> {
    pub genesis_hash: Hash,
    pub dagknight_store: Arc<C>,
    pub headers_store: Arc<O>,
    pub relations_store: Arc<RwLock<D>>,
    pub reachability_service: MTReachabilityService<R>,
    pub counters: Arc<DagknightCounters>,
}

impl<
    C: DagknightStore + DagknightStoreReader,
    O: HeaderStoreReader + 'static,
    D: RelationsStoreReader + Clone,
    R: ReachabilityStoreReader + Clone,
> DagknightExecutor<C, O, D, R>
{
    pub fn new(
        genesis_hash: Hash,
        dagknight_store: Arc<C>,
        headers_store: Arc<O>,
        relations_store: Arc<RwLock<D>>,
        reachability_service: MTReachabilityService<R>,
    ) -> Self {
        Self {
            genesis_hash,
            dagknight_store,
            headers_store,
            relations_store,
            reachability_service,
            counters: Arc::new(DagknightCounters::new()),
        }
    }

    pub fn dagknight(&self, parents: &[Hash]) -> DagknightData {
        assert!(!parents.is_empty(), "parents cannot be empty in dagknight");

        self.counters.total_calls.fetch_add(1, Ordering::Relaxed);

        if parents.len() == 1 {
            return DagknightData { selected_parent: parents[0], conflict_ordered_parents: Vec::new() };
        }

        let mut conflict_genesis = self.common_chain_ancestor(parents);
        let mut curr_subgroup = Arc::new(parents.iter().unique().copied().collect_vec());
        let mut conflict_ordered_parents = Vec::new();
        debug!("DAGKNIGHT: conflict_genesis: {:#?}", conflict_genesis);

        while curr_subgroup.len() > 1 {
            let agreement_grouping: HashMap<Hash, Arc<Vec<Hash>>> = curr_subgroup
                .iter()
                .copied()
                .into_group_map_by(|&parent| self.reachability_service.get_next_chain_ancestor(parent, conflict_genesis))
                .into_iter()
                .map(|(k, v)| (k, Arc::new(v)))
                .collect();

            if agreement_grouping.len() == 1 {
                let (_, subgroup) = agreement_grouping.into_iter().next().unwrap();
                curr_subgroup = subgroup;
                if curr_subgroup.len() <= 1 {
                    break;
                }
                let next_conflict_genesis = self.common_chain_ancestor(&curr_subgroup);
                if next_conflict_genesis == conflict_genesis {
                    break;
                }
                conflict_genesis = next_conflict_genesis;
                continue;
            }

            let (winning_nca, winning_subgroup) = {
                let best_groups = self.rank(conflict_genesis, &agreement_grouping, &curr_subgroup);

                if best_groups.len() > 1 {
                    self.tie_breaking(conflict_genesis, &curr_subgroup, &best_groups)
                } else if let Some(single_winner) = best_groups.into_iter().next() {
                    (single_winner.conflict_genesis, single_winner.subgroup)
                } else {
                    let first_entry = agreement_grouping.iter().next().unwrap();
                    (*first_entry.0, first_entry.1.clone())
                }
            };

            for (&nca_hash, subgroup) in agreement_grouping.iter() {
                if nca_hash != winning_nca {
                    conflict_ordered_parents.extend(subgroup.as_ref().iter().copied());
                }
            }

            curr_subgroup = winning_subgroup;
            conflict_genesis = self.common_chain_ancestor(&curr_subgroup);
        }

        assert_eq!(1, curr_subgroup.len(), "Expected dagknight to resolve to a single selected parent");

        conflict_ordered_parents.reverse();
        debug!("DAGKNIGHT: sp: {} | conflict_ordered_parents: {:?}", curr_subgroup[0], conflict_ordered_parents);

        DagknightData { selected_parent: curr_subgroup[0], conflict_ordered_parents }
    }

    pub fn common_chain_ancestor(&self, parents: &[Hash]) -> Hash {
        let start = parents[0];
        if start == self.genesis_hash {
            return self.genesis_hash;
        }

        for cb in self.reachability_service.default_backward_chain_iterator(start) {
            if parents[1..].iter().all(|&p| self.reachability_service.is_chain_ancestor_of(cb, p)) {
                return cb;
            }
        }

        self.genesis_hash
    }

    pub fn rank(
        &self,
        conflict_genesis: Hash,
        agreement_grouping: &HashMap<Hash, Arc<Vec<Hash>>>,
        curr_subgroup: &[Hash],
    ) -> Vec<GroupMetadata> {
        let mut group_results = Vec::new();
        let cg_work = if conflict_genesis == self.genesis_hash {
            BlueWorkType::from(1u64)
        } else {
            calc_work(self.headers_store.get_bits(conflict_genesis).unwrap_or(0))
        };
        let cg_block = BlockWithWork::new(conflict_genesis, cg_work);

        for (&nca_hash, subgroup) in agreement_grouping {
            let reachability_service = self.reachability_service.clone();
            let relations_store = self.relations_store.read().clone();
            let relations_service = FutureIntersectRelations::new(relations_store, reachability_service.clone(), conflict_genesis);

            let search_result = RankSearcher::search(|k| {
                let conflict_zone_manager = ConflictZoneManager::with_free_search(
                    k,
                    conflict_genesis,
                    self.dagknight_store.clone(),
                    self.headers_store.clone(),
                    relations_service.clone(),
                    reachability_service.clone(),
                    false,
                );

                let subgroup_sp = conflict_zone_manager.find_selected_parent(subgroup.iter().copied());
                conflict_zone_manager.fill_zone_data(curr_subgroup, Some(nca_hash));

                let virtual_gd = conflict_zone_manager.k_colouring(curr_subgroup, k, Some(subgroup_sp));

                let mut cascade = CascadeMaintainer::new(cg_block, k);

                for &blue in virtual_gd.mergeset_blues.iter() {
                    if blue != conflict_genesis {
                        let work = calc_work(self.headers_store.get_bits(blue).unwrap_or(0));
                        cascade.add_blue(BlockWithWork::new(blue, work), &reachability_service);
                    }
                }

                for &red in virtual_gd.mergeset_reds.iter() {
                    if red != conflict_genesis {
                        let work = calc_work(self.headers_store.get_bits(red).unwrap_or(0));
                        cascade.add_red(BlockWithWork::new(red, work), &reachability_service);
                    }
                }

                if cascade.is_valid() {
                    let sp_work = self.headers_store.get_bits(subgroup_sp).map(calc_work).unwrap_or_else(|_| BlueWorkType::from(0u64));
                    Some(SortableBlock { hash: subgroup_sp, blue_work: sp_work })
                } else {
                    None
                }
            });

            if let Some(res) = search_result {
                group_results.push(GroupMetadata {
                    conflict_genesis: nca_hash,
                    subgroup: subgroup.clone(),
                    k: res.k,
                    selected_parent: res.result,
                });
            }
        }

        if group_results.is_empty() {
            return Vec::new();
        }

        let min_k = group_results.iter().map(|g| g.k).min().unwrap();
        group_results.into_iter().filter(|g| g.k == min_k).collect()
    }

    pub fn tie_breaking(
        &self,
        conflict_genesis: Hash,
        all_tips: &[Hash],
        best_groups: &[GroupMetadata],
    ) -> (Hash, Arc<Vec<Hash>>) {
        let tie_breaker = DagknightTieBreaker::new(
            self.dagknight_store.clone(),
            self.headers_store.clone(),
            self.relations_store.clone(),
            self.reachability_service.clone(),
        );

        let context = TieBreakContext {
            conflict_genesis,
            all_tips,
            subgroups: best_groups,
            k: best_groups[0].k,
        };

        let winning_index = tie_breaker.tie_break(&context);
        (best_groups[winning_index].conflict_genesis, best_groups[winning_index].subgroup.clone())
    }
}
