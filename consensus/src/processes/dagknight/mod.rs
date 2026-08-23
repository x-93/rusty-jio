use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use jio_consensus_core::KType;
use jio_hashes::Hash;

use crate::processes::ghostdag::ordering::SortableBlock;

mod appendable_segment_tree_api;
mod appendable_segment_tree_impl;
pub use appendable_segment_tree_api::{bucket_for_score, AppendableSegmentTreeApi, Bucket};
pub use appendable_segment_tree_impl::AppendableSegmentTree;
pub mod manager;
pub mod protocol;
pub mod rank_search;
pub mod tie_breaking;
pub mod umc_cascade;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct GroupMetadata {
    pub conflict_genesis: Hash,
    pub subgroup: Arc<Vec<Hash>>,
    pub k: KType,
    pub selected_parent: SortableBlock,
}

/// UMC cascade voting performance counters.
pub struct DagknightCounters {
    pub total_calls: AtomicU64,
    pub total_voting_blocks: AtomicU64,
    pub total_cascade_flips: AtomicU64,
    pub max_cascade_flips: AtomicU64,
}

impl Default for DagknightCounters {
    fn default() -> Self {
        Self::new()
    }
}

impl DagknightCounters {
    pub fn new() -> Self {
        Self {
            total_calls: AtomicU64::new(0),
            total_voting_blocks: AtomicU64::new(0),
            total_cascade_flips: AtomicU64::new(0),
            max_cascade_flips: AtomicU64::new(0),
        }
    }
}
