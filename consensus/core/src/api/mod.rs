pub mod args;
pub mod counters;
pub mod stats;

pub use args::*;
pub use counters::*;
pub use stats::*;

use crate::block::{Block, BlockTemplate};
use crate::coinbase::MinerData;
use crate::errors::BlockProcessResult;
use jio_hashes::Hash;

/// The primary consensus service trait exposed to other node subsystems.
pub trait ConsensusApi: Send + Sync {
    /// Validates and inserts a new block into the BlockDAG.
    fn validate_and_insert_block(&self, block: Block) -> BlockProcessResult<()>;

    /// Builds a new block template for miners given miner coinbase data.
    fn build_block_template(&self, miner_data: MinerData) -> BlockProcessResult<BlockTemplate>;

    /// Returns the current selected parent (sink) of the DAG.
    fn get_sink(&self) -> Hash;
}
