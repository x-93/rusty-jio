use crate::model::stores::reachability::{Interval, MemoryReachabilityStore};
use jio_consensus_core::blockhash;
use jio_hashes::Hash;
use std::collections::HashMap;

pub struct TreeBuilder<'a> {
    store: &'a mut MemoryReachabilityStore,
    root: Option<Hash>,
    root_interval: Interval,
    tree_parents: HashMap<Hash, Hash>,
    tree_children: HashMap<Hash, Vec<Hash>>,
}

impl<'a> TreeBuilder<'a> {
    pub fn new(store: &'a mut MemoryReachabilityStore) -> Self {
        Self {
            store,
            root: None,
            root_interval: Interval::maximal(),
            tree_parents: HashMap::new(),
            tree_children: HashMap::new(),
        }
    }

    pub fn init_with_params(&mut self, root: Hash, interval: Interval) -> &mut Self {
        self.root = Some(root);
        self.root_interval = interval;
        self.store.insert(root, blockhash::ORIGIN, interval);
        self
    }

    pub fn add_block(&mut self, child: Hash, parent: Hash) -> &mut Self {
        self.tree_parents.insert(child, parent);
        self.tree_children.entry(parent).or_default().push(child);
        self.store.insert(child, parent, Interval::empty());
        self.recompute_intervals();
        self
    }

    fn recompute_intervals(&mut self) {
        if let Some(root) = self.root {
            let mut current_time = self.root_interval.start;
            self.dfs_assign(root, &mut current_time);
        }
    }

    fn dfs_assign(&mut self, node: Hash, current_time: &mut u64) {
        let start = *current_time;
        if let Some(children) = self.tree_children.get(&node).cloned() {
            for child in children {
                *current_time += 1;
                self.dfs_assign(child, current_time);
            }
        }
        let end = *current_time;
        let interval = Interval::new(start, end);
        self.store.set_interval(node, interval);
    }
}
