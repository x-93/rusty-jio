//! Consensus pipeline performance counters.

#[derive(Clone, Debug, Default)]
pub struct ConsensusCounters {
    pub blocks_processed: u64,
    pub headers_processed: u64,
    pub txs_processed: u64,
}
