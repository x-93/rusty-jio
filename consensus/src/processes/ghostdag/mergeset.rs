use crate::model::services::reachability::ReachabilityService;
use crate::model::stores::ghostdag::GhostdagData;
use crate::model::stores::relations::RelationsStore;
use jio_consensus_core::blockhash::BlockHash;
use std::collections::{HashSet, VecDeque};

pub fn get_mergeset(data: &GhostdagData) -> Vec<BlockHash> {
    data.mergeset().copied().collect()
}

/// Finds all candidate blocks in past(parents) \ past(selected_parent).
pub fn find_mergeset_candidates(
    parents: &[BlockHash],
    selected_parent: BlockHash,
    relations: &RelationsStore,
    reachability: &(impl ReachabilityService + ?Sized),
) -> Vec<BlockHash> {
    let mut candidates = Vec::new();
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    for &parent in parents {
        if parent != selected_parent && visited.insert(parent) {
            queue.push_back(parent);
        }
    }

    while let Some(current) = queue.pop_front() {
        if reachability.is_dag_ancestor_of(current, selected_parent) {
            continue;
        }

        candidates.push(current);

        if let Some(parent_list) = relations.get_parents(&current) {
            for p in parent_list {
                if p != selected_parent && visited.insert(p) {
                    queue.push_back(p);
                }
            }
        }
    }

    candidates
}
