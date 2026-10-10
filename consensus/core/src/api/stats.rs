use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConsensusStats {
    pub block_count: u64,
    pub header_count: u64,
    pub tip_hashes: Vec<jio_hashes::Hash>,
    pub difficulty: f64,
    pub past_median_time: u64,
}