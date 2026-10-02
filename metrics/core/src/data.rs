//! Node metric data structures and snapshots.

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct NodeMetricsSnapshot {
    /// Blocks per second over the last sampling window
    pub bps: f64,
    /// Transactions per second over the last sampling window
    pub tps: f64,
    /// Current virtual selected parent blue score
    pub blue_score: u64,
    /// Current virtual block DAA score
    pub daa_score: u64,
    /// Number of transactions currently in the mempool
    pub mempool_size: usize,
    /// Resident set size (physical RAM) in bytes
    pub resident_set_size_bytes: u64,
    /// Number of active peer connections
    pub peer_count: usize,
}
