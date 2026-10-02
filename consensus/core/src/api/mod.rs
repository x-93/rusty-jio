pub mod args;
pub mod counters;
pub mod stats;

pub use args::*;
pub use counters::*;
pub use stats::*;

use std::sync::Arc;
use jio_hashes::Hash;
use crate::block::Block;
use crate::header::Header;
use crate::coinbase::MinerData;
use crate::tx::Transaction;
use crate::blockstatus::BlockStatus;
use crate::errors::consensus::RuleError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockValidationMode {
    SkipAll,
    HeaderOnly,
    Full,
}

#[derive(Clone, Debug)]
pub struct BlockTemplate {
    pub block: Block,
    pub coinbase_has_red_reward: bool,
    pub selected_parent_hash: Hash,
}

pub trait TxSelector: Send {
    fn select_transactions(&mut self) -> Vec<Transaction>;
}

pub type BlockProcessResult<T> = Result<T, RuleError>;

pub trait ConsensusApi: Send + Sync {
    fn build_block_template(&self, miner_data: MinerData, tx_selector: Box<dyn TxSelector>) -> Result<BlockTemplate, RuleError>;
    fn validate_and_insert_block(&self, block: Block) -> BlockProcessResult<BlockStatus>;
    fn validate_and_insert_trusted_block(&self, tb: crate::trusted::TrustedBlock) -> BlockProcessResult<BlockStatus>;
    fn get_block(&self, hash: Hash) -> Result<Block, RuleError>;
    fn get_header(&self, hash: Hash) -> Result<Arc<Header>, RuleError>;
    fn get_block_status(&self, hash: Hash) -> Option<BlockStatus>;
    fn get_virtual_daa_score(&self) -> u64;
    fn get_virtual_bits(&self) -> u32;
    fn get_virtual_past_median_time(&self) -> u64;
    fn get_virtual_parents(&self) -> Arc<Vec<Hash>>;
    fn get_tips(&self) -> Arc<Vec<Hash>>;
    fn get_stats(&self) -> ConsensusStats;
    fn estimate_network_hashes_per_second(&self, start_hash: Option<Hash>, window_size: usize) -> Result<u64, RuleError>;
    fn calculate_transaction_mass(&self, tx: &Transaction) -> u64;
}
