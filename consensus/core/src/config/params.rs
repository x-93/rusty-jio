//! Consensus parameters and network constants.

use super::super::network::NetworkType;
use super::genesis::{get_genesis, GenesisBlock};

#[derive(Clone, Debug)]
pub struct Params {
    pub dns_seeders: &'static [&'static str],
    pub net: NetworkType,
    pub name: &'static str,
    pub genesis: GenesisBlock,
    pub ghostdag_k: u64,
    pub target_time_per_block: u64,
    pub max_block_mass: u64,
    pub max_block_parents: u8,
    pub difficulty_window_size: usize,
    pub coinbase_maturity: u64,
    pub pruning_depth: u64,
    pub finality_depth: u64,
}

impl Params {
    pub fn bps(&self) -> u64 {
        1000 / self.target_time_per_block
    }

    pub fn mainnet() -> Self {
        Self {
            dns_seeders: &["seed.jio.network", "seed2.jio.network"],
            net: NetworkType::Mainnet,
            name: "jio-mainnet",
            genesis: get_genesis(NetworkType::Mainnet),
            ghostdag_k: 18,
            target_time_per_block: 1000, // 1 BPS
            max_block_mass: 500_000,
            max_block_parents: 10,
            difficulty_window_size: 2641,
            coinbase_maturity: 100,
            pruning_depth: 185_798,
            finality_depth: 86_400,
        }
    }

    pub fn testnet() -> Self {
        Self {
            dns_seeders: &["testnet-seed.jio.network"],
            net: NetworkType::Testnet,
            name: "jio-testnet",
            genesis: get_genesis(NetworkType::Testnet),
            ghostdag_k: 32,
            target_time_per_block: 100, // 10 BPS
            max_block_mass: 500_000,
            max_block_parents: 16,
            difficulty_window_size: 2641,
            coinbase_maturity: 100,
            pruning_depth: 185_798,
            finality_depth: 86_400,
        }
    }

    pub fn simnet() -> Self {
        Self {
            dns_seeders: &[],
            net: NetworkType::Simnet,
            name: "jio-simnet",
            genesis: get_genesis(NetworkType::Simnet),
            ghostdag_k: 18,
            target_time_per_block: 1000,
            max_block_mass: 500_000,
            max_block_parents: 10,
            difficulty_window_size: 140,
            coinbase_maturity: 10,
            pruning_depth: 1000,
            finality_depth: 500,
        }
    }

    pub fn devnet() -> Self {
        Self {
            dns_seeders: &[],
            net: NetworkType::Devnet,
            name: "jio-devnet",
            genesis: get_genesis(NetworkType::Devnet),
            ghostdag_k: 18,
            target_time_per_block: 1000,
            max_block_mass: 500_000,
            max_block_parents: 10,
            difficulty_window_size: 2641,
            coinbase_maturity: 100,
            pruning_depth: 185_798,
            finality_depth: 86_400,
        }
    }
}
