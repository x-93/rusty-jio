use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusStats {
    pub virtual_daa_score: u64,
    pub virtual_blue_score: u64,
    pub past_median_time: u64,
}
