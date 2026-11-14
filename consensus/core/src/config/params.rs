use crate::KType;
use crate::config::constants::consensus::*;
use crate::config::genesis::{GenesisBlock, create_genesis_header};
use crate::network::NetworkId;
use jio_math::Uint256;

#[derive(Clone, Debug)]
pub struct Params {
    pub net: NetworkId,
    pub genesis: GenesisBlock,
    pub ghostdag_k: KType,
    pub timestamp_deviation_tolerance: u64,
    pub target_time_per_block: u64,
    pub max_difficulty_target: Uint256,
    pub max_block_mass: u64,
}

impl Params {
    pub fn new(
        net: NetworkId,
        genesis: GenesisBlock,
        ghostdag_k: KType,
        target_time_per_block: u64,
        max_difficulty_target: Uint256,
        max_block_mass: u64,
    ) -> Self {
        Self {
            net,
            genesis,
            ghostdag_k,
            timestamp_deviation_tolerance: NEW_TIMESTAMP_DEVIATION_TOLERANCE,
            target_time_per_block,
            max_difficulty_target,
            max_block_mass,
        }
    }

    #[inline(always)]
    pub fn bps(&self) -> u64 {
        1000 / self.target_time_per_block.max(1)
    }

    #[inline(always)]
    pub fn difficulty_window_size(&self, _daa_score: u64) -> usize {
        LEGACY_DIFFICULTY_WINDOW_SIZE
    }
}

pub fn default_mainnet_params() -> Params {
    let header = create_genesis_header(1, 1700000000, 0x1e7fffff);
    Params::new(
        NetworkId::new(crate::network::NetworkType::Mainnet),
        GenesisBlock::new(header),
        LEGACY_DEFAULT_GHOSTDAG_K,
        1000, // 1 block per second = 1000 ms
        MAX_DIFFICULTY_TARGET,
        500_000,
    )
}
