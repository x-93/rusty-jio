use crate::counters::PerformanceCounters;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct PerformanceMonitor {
    pub counters: Arc<PerformanceCounters>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(PerformanceCounters::new()),
        }
    }

    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.counters.blocks_processed.load(std::sync::atomic::Ordering::Relaxed),
            self.counters.transactions_processed.load(std::sync::atomic::Ordering::Relaxed),
            self.counters.header_processed_count.load(std::sync::atomic::Ordering::Relaxed),
        )
    }
}
