use crate::model::services::reachability::{MTReachabilityService, ReachabilityService};
use crate::model::stores::ghostdag::{GhostdagData, GhostdagStore};
use crate::model::stores::reachability::ReachabilityStore;
use crate::model::stores::relations::RelationsStore;
use crate::processes::ghostdag::mergeset::find_mergeset_candidates;
use crate::processes::ghostdag::ordering::sort_blocks_topological;
use jio_consensus_core::blockhash::{BlockHash, ORIGIN};
use jio_consensus_core::{BlockHashMap, HashMapCustomHasher, KType};
use jio_math::Uint192;
use std::sync::Arc;

#[derive(Clone)]
pub struct GhostdagManager {
    k: u64,
    ghostdag_store: GhostdagStore,
    relations: RelationsStore,
    reachability: MTReachabilityService<ReachabilityStore>,
}

impl GhostdagManager {
    pub fn new(
        k: u64,
        ghostdag_store: GhostdagStore,
        relations: RelationsStore,
        reachability: MTReachabilityService<ReachabilityStore>,
    ) -> Self {
        Self {
            k,
            ghostdag_store,
            relations,
            reachability,
        }
    }

    pub fn ghostdag(&self, parents: &[BlockHash]) -> GhostdagData {
        if parents.is_empty() {
            return GhostdagData::new(0, Uint192::ZERO, ORIGIN, vec![], vec![]);
        }

        // 1. Elect selected parent having maximum blue work (tie-breaking by blue score and hash)
        let mut selected_parent = parents[0];
        let mut max_work = self
            .ghostdag_store
            .get_blue_work(&selected_parent)
            .unwrap_or(Uint192::ZERO);
        let mut max_score = self
            .ghostdag_store
            .get_blue_score(&selected_parent)
            .unwrap_or(0);

        for &parent in &parents[1..] {
            let parent_work = self
                .ghostdag_store
                .get_blue_work(&parent)
                .unwrap_or(Uint192::ZERO);
            let parent_score = self
                .ghostdag_store
                .get_blue_score(&parent)
                .unwrap_or(0);

            if parent_work > max_work
                || (parent_work == max_work
                    && (parent_score > max_score
                        || (parent_score == max_score && parent < selected_parent)))
            {
                selected_parent = parent;
                max_work = parent_work;
                max_score = parent_score;
            }
        }

        let selected_parent_data = self
            .ghostdag_store
            .get_data(&selected_parent)
            .unwrap_or_else(|| {
                Arc::new(GhostdagData::new(0, Uint192::ZERO, ORIGIN, vec![], vec![]))
            });

        // 2. Discover all unmerged candidate blocks in past(parents) \ past(selected_parent)
        let mut candidates = find_mergeset_candidates(
            parents,
            selected_parent,
            &self.relations,
            &self.reachability,
        );

        // 3. Sort candidates topologically by GhostDAG ordering (blue_work, blue_score, hash)
        sort_blocks_topological(&mut candidates, &self.ghostdag_store);

        // 4. Greedy K-cluster coloring
        let mut mergeset_blues = Vec::new();
        let mut mergeset_reds = Vec::new();
        let mut blues_anticone_sizes: BlockHashMap<KType> = BlockHashMap::new();

        // Copy existing anticone size estimates from selected parent data
        for (blue, &size) in &selected_parent_data.blues_anticone_sizes {
            blues_anticone_sizes.insert(*blue, size);
        }

        for candidate in candidates {
            // Find all blue blocks in candidate's anticone
            let mut anticone_blues = Vec::new();

            // Check against newly colored mergeset blues
            for &blue in &mergeset_blues {
                if !self.reachability.is_dag_ancestor_of(blue, candidate)
                    && !self.reachability.is_dag_ancestor_of(candidate, blue)
                {
                    anticone_blues.push(blue);
                }
            }

            // Check against selected parent's mergeset blues
            for &blue in &selected_parent_data.mergeset_blues {
                if !self.reachability.is_dag_ancestor_of(blue, candidate)
                    && !self.reachability.is_dag_ancestor_of(candidate, blue)
                {
                    anticone_blues.push(blue);
                }
            }

            // Candidate can be blue if its anticone size <= K and all anticone blues have size < K
            let candidate_anticone_valid = (anticone_blues.len() as u64) <= self.k;
            let anticone_blues_valid = anticone_blues.iter().all(|b| {
                let current_size = blues_anticone_sizes.get(b).copied().unwrap_or(0);
                (current_size as u64) < self.k
            });

            if candidate_anticone_valid && anticone_blues_valid {
                // Color candidate as BLUE
                mergeset_blues.push(candidate);
                blues_anticone_sizes.insert(candidate, anticone_blues.len() as KType);

                // Increment anticone sizes for mutually anticone blue blocks
                for b in anticone_blues {
                    let entry = blues_anticone_sizes.entry(b).or_insert(0);
                    *entry = entry.saturating_add(1);
                }
            } else {
                // Color candidate as RED
                mergeset_reds.push(candidate);
            }
        }

        // 5. Calculate accumulated blue score and blue work
        let blue_score = selected_parent_data.blue_score + (mergeset_blues.len() as u64) + 1;
        let blue_work = selected_parent_data.blue_work
            + Uint192::from(mergeset_blues.len() as u64 + 1);

        GhostdagData::new_with_anticone_sizes(
            blue_score,
            blue_work,
            selected_parent,
            mergeset_blues,
            mergeset_reds,
            blues_anticone_sizes,
        )
    }
}
