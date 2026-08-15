use jio_consensus_core::blockhash::BlockHash;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct ReachabilityTree {
    intervals: HashMap<BlockHash, (u64, u64)>,
}

impl ReachabilityTree {
    pub fn new() -> Self {
        Self {
            intervals: HashMap::new(),
        }
    }

    pub fn set_interval(&mut self, hash: BlockHash, start: u64, end: u64) {
        self.intervals.insert(hash, (start, end));
    }

    pub fn is_ancestor_of(&self, ancestor: &BlockHash, descendant: &BlockHash) -> bool {
        if let (Some(&(a_start, a_end)), Some(&(d_start, d_end))) = (
            self.intervals.get(ancestor),
            self.intervals.get(descendant),
        ) {
            a_start <= d_start && a_end >= d_end
        } else {
            false
        }
    }
}
