use crate::api::counters::ConsensusCountersSnapshot;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsensusStats {
    pub counters: ConsensusCountersSnapshot,
    pub virtual_daa_score: u64,
    pub past_median_time: u64,
}
