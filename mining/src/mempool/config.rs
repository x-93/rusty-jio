use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MempoolConfig {
    pub maximum_transaction_count: usize,
    pub maximum_orphan_transaction_count: usize,
    pub minimum_relay_fee_rate: f64,
    pub maximum_block_mass: u64,
    pub transaction_expire_scan_interval_milliseconds: u64,
    pub transaction_expire_time_milliseconds: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            maximum_transaction_count: 50_000,
            maximum_orphan_transaction_count: 10_000,
            minimum_relay_fee_rate: 1.0, // 1 sompi per mass unit
            maximum_block_mass: 500_000,
            transaction_expire_scan_interval_milliseconds: 10_000,
            transaction_expire_time_milliseconds: 24 * 60 * 60 * 1000, // 24 hours
        }
    }
}
