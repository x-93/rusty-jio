use super::bps::Bps;
use jio_hashes::Hash;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Simnet,
    Devnet,
}

#[derive(Clone, Debug)]
pub struct Params {
    pub net: NetworkType,
    pub bps: Bps,
    pub ghostdag_k: u64,
    pub target_time_per_block_ms: u64,
    pub difficulty_window_size: usize,
    pub min_difficulty_window_size: usize,
    pub mergeset_size_limit: u64,
    pub finality_depth: u64,
    pub pruning_depth: u64,
    pub coinbase_maturity: u64,
    pub max_block_mass: u64,
    pub genesis_hash: Hash,
}

impl Params {
    pub fn mainnet() -> Self {
        let bps = Bps::Bps10;
        Self {
            net: NetworkType::Mainnet,
            bps,
            ghostdag_k: bps.ghostdag_k(),
            target_time_per_block_ms: bps.target_time_per_block_ms(),
            difficulty_window_size: bps.difficulty_window_size(),
            min_difficulty_window_size: 263,
            mergeset_size_limit: bps.ghostdag_k() * 10,
            finality_depth: 86_400,
            pruning_depth: 185_798,
            coinbase_maturity: 100,
            max_block_mass: 500_000,
            genesis_hash: Hash::ZERO,
        }
    }

    pub fn testnet() -> Self {
        let mut p = Self::mainnet();
        p.net = NetworkType::Testnet;
        p
    }

    pub fn simnet() -> Self {
        let bps = Bps::Bps1;
        Self {
            net: NetworkType::Simnet,
            bps,
            ghostdag_k: bps.ghostdag_k(),
            target_time_per_block_ms: bps.target_time_per_block_ms(),
            difficulty_window_size: 140,
            min_difficulty_window_size: 10,
            mergeset_size_limit: bps.ghostdag_k() * 10,
            finality_depth: 1000,
            pruning_depth: 2000,
            coinbase_maturity: 10,
            max_block_mass: 500_000,
            genesis_hash: Hash::ZERO,
        }
    }
}
