use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct ConsensusCounters {
    pub blocks_submitted: Arc<AtomicU64>,
    pub header_counts: Arc<AtomicU64>,
    pub body_counts: Arc<AtomicU64>,
    pub txs_processed: Arc<AtomicU64>,
}

impl ConsensusCounters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> ConsensusCountersSnapshot {
        ConsensusCountersSnapshot {
            blocks_submitted: self.blocks_submitted.load(Ordering::Relaxed),
            header_counts: self.header_counts.load(Ordering::Relaxed),
            body_counts: self.body_counts.load(Ordering::Relaxed),
            txs_processed: self.txs_processed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsensusCountersSnapshot {
    pub blocks_submitted: u64,
    pub header_counts: u64,
    pub body_counts: u64,
    pub txs_processed: u64,
}
