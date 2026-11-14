use crate::model::stores::relations::RelationsStoreReader;
use crate::processes::reachability::ReachabilityService;
use jio_hashes::Hash;
use std::collections::{HashSet, VecDeque};

/// Calculates the merge set of a new block with the given parents and selected parent:
/// MergeSet = (Past(parents) \setminus Past(selected_parent)) \cup (parents \setminus {selected_parent})
pub fn calculate_mergeset(
    relations: &impl RelationsStoreReader,
    reachability: &impl ReachabilityService,
    parents: &[Hash],
    selected_parent: Hash,
) -> Vec<Hash> {
    if parents.is_empty() {
        return Vec::new();
    }

    let mut mergeset = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Start traversal from non-selected parents that are not already ancestors of selected_parent
    for &parent in parents {
        if parent != selected_parent {
            if !reachability.is_dag_ancestor_of(parent, selected_parent) {
                if visited.insert(parent) {
                    mergeset.push(parent);
                    queue.push_back(parent);
                }
            } else {
                visited.insert(parent);
            }
        }
    }

    // Traverse ancestors of the non-selected parents
    while let Some(current) = queue.pop_front() {
        if let Ok(current_parents) = relations.get_parents(current) {
            for &parent in current_parents.iter() {
                if !visited.contains(&parent) {
                    visited.insert(parent);
                    if !reachability.is_dag_ancestor_of(parent, selected_parent) {
                        mergeset.push(parent);
                        queue.push_back(parent);
                    }
                }
            }
        }
    }

    mergeset
}
