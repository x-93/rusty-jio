use crate::model::stores::reachability::{Interval, ReachabilityStore};
use crate::model::stores::relations::RelationsStore;
use jio_consensus_core::blockhash::BlockHash;
use std::collections::{HashSet, VecDeque};

#[derive(Clone)]
pub struct ReachabilityService {
    relations: RelationsStore,
    reachability: ReachabilityStore,
}

impl ReachabilityService {
    pub fn new(relations: RelationsStore, reachability: ReachabilityStore) -> Self {
        Self {
            relations,
            reachability,
        }
    }

    pub fn init_genesis(&self, genesis: BlockHash) {
        self.reachability
            .insert(genesis, genesis, Interval::maximal());
    }

    pub fn add_block(&self, hash: BlockHash, selected_parent: BlockHash) {
        if let Some(parent_interval) = self.reachability.get_interval(&selected_parent) {
            let existing_children_count = self
                .reachability
                .get_children(&selected_parent)
                .map(|c| c.len())
                .unwrap_or(0) as u64;

            let parent_size = parent_interval.size();
            let chunk_size = (parent_size / 65536).max(1);

            let child_start = parent_interval
                .start
                .saturating_add(existing_children_count.saturating_mul(chunk_size))
                .saturating_add(1);
            let child_end = child_start.saturating_add(chunk_size).min(parent_interval.end.saturating_sub(1));

            let child_interval = if child_start < child_end {
                Interval::new(child_start, child_end)
            } else {
                Interval::new(parent_interval.start.saturating_add(1), parent_interval.end.saturating_sub(1))
            };

            self.reachability
                .insert(hash, selected_parent, child_interval);
        }
    }

    pub fn is_tree_ancestor_of(&self, ancestor: &BlockHash, descendant: &BlockHash) -> bool {
        if ancestor == descendant {
            return true;
        }
        if let (Some(a_interval), Some(d_interval)) = (
            self.reachability.get_interval(ancestor),
            self.reachability.get_interval(descendant),
        ) {
            a_interval.start < d_interval.start && d_interval.end < a_interval.end
        } else {
            false
        }
    }

    pub fn is_dag_ancestor_of(&self, ancestor: &BlockHash, descendant: &BlockHash) -> bool {
        if ancestor == descendant {
            return true;
        }

        // Fast path: tree interval strict containment
        if self.is_tree_ancestor_of(ancestor, descendant) {
            return true;
        }

        // Fallback path: DAG traversal
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back(*descendant);
        visited.insert(*descendant);

        while let Some(current) = queue.pop_front() {
            if let Some(parents) = self.relations.get_parents(&current) {
                for parent in parents {
                    if parent == *ancestor {
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
}
