use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default, Debug)]
pub struct ConsensusCounters {
    pub blocks_submitted: AtomicU64,
    pub headers_processed: AtomicU64,
    pub transactions_processed: AtomicU64,
}

impl ConsensusCounters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> ConsensusCountersSnapshot {
        ConsensusCountersSnapshot {
            blocks_submitted: self.blocks_submitted.load(Ordering::Relaxed),
            headers_processed: self.headers_processed.load(Ordering::Relaxed),
            transactions_processed: self.transactions_processed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConsensusCountersSnapshot {
    pub blocks_submitted: u64,
    pub headers_processed: u64,
    pub transactions_processed: u64,
}
