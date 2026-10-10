use super::genesis::GenesisBlock;
use crate::network::NetworkId;
use crate::KType;
use jio_math::Uint256;
use serde::Serialize;

pub const MAINNET_BPS: u64 = 1;
pub const TESTNET_BPS: u64 = 10;

#[derive(Clone, Debug, Serialize)]
pub struct Params {
    pub dns_seeders: &'static [&'static str],
    pub net: NetworkId,
    pub name: &'static str,
    pub genesis: GenesisBlock,
    pub ghostdag_k: KType,
    pub timestamp_deviation_tolerance: u64,
    pub target_time_per_block: u64,
    pub max_block_mass: u64,
    pub max_difficulty_target: Uint256,
    pub min_difficulty_window_len: usize,
    pub difficulty_window_duration: u64,
    pub coinbase_maturity: u64,
    pub finality_depth: u64,
    pub merge_depth: u64,
    pub pruning_proof_m: u64,
    pub bps: u64,
}

impl Params {
    pub fn bps(&self) -> u64 {
        self.bps
    }
}
