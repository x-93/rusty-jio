use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct ConsensusCounters {
    pub blocks_submitted: AtomicU64,
    pub header_counts: AtomicU64,
    pub body_counts: AtomicU64,
    pub txs_counts: AtomicU64,
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
            txs_counts: self.txs_counts.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ConsensusCountersSnapshot {
    pub blocks_submitted: u64,
    pub header_counts: u64,
    pub body_counts: u64,
    pub txs_counts: u64,
}