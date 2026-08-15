use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct PerformanceCounters {
    pub blocks_processed: AtomicU64,
    pub transactions_processed: AtomicU64,
    pub header_processed_count: AtomicU64,
    pub total_block_processing_time_ms: AtomicU64,
}

impl PerformanceCounters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_block(&self, tx_count: u64, duration_ms: u64) {
        self.blocks_processed.fetch_add(1, Ordering::Relaxed);
        self.transactions_processed.fetch_add(tx_count, Ordering::Relaxed);
        self.total_block_processing_time_ms.fetch_add(duration_ms, Ordering::Relaxed);
    }

    pub fn record_header(&self) {
        self.header_processed_count.fetch_add(1, Ordering::Relaxed);
    }
}
