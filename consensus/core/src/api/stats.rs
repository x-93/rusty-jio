//! BlockDAG consensus statistics.

#[derive(Clone, Debug, Default)]
pub struct ConsensusStats {
    pub num_blocks: u64,
    pub num_headers: u64,
    pub num_tips: usize,
    pub virtual_daa_score: u64,
    pub virtual_blue_score: u64,
}
