pub mod args;
pub mod counters;
pub mod stats;

pub use args::*;
pub use counters::*;
pub use stats::*;

use crate::block::{Block, BlockTemplate};
use crate::coinbase::MinerData;
use crate::errors::consensus::ConsensusError;

pub trait ConsensusApi: Send + Sync {
    fn build_block_template(&self, miner_data: MinerData) -> Result<BlockTemplate, ConsensusError>;
    fn validate_and_insert_block(&self, block: Block) -> Result<(), ConsensusError>;
    fn get_stats(&self) -> ConsensusStats;
}