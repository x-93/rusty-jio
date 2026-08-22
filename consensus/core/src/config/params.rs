use super::constants::*;
use super::genesis::GenesisBlock;
use crate::KType;
use crate::network::{NetworkId, NetworkType};
use jio_math::Uint256;

/// Complete consensus parameters governing a specific Jio network.
#[derive(Clone, Debug)]
pub struct Params {
    pub net: NetworkId,
    pub name: &'static str,
    pub genesis: GenesisBlock,
    pub ghostdag_k: KType,
    pub target_time_per_block: u64,
    pub max_block_mass: u64,
    pub finality_depth: u64,
    pub pruning_depth: u64,
    pub min_difficulty_bits: u32,
    pub max_difficulty_target: Uint256,
    pub difficulty_window_size: usize,
    pub coinbase_maturity: u64,
}

impl Params {
    pub fn mainnet() -> Self {
        let genesis = GenesisBlock::build_genesis_block(
            1700000000000,
            0x1e7fffff,
            b"Jio Mainnet Genesis: Decentralized High-Throughput BlockDAG".to_vec(),
        );

        Self {
            net: NetworkId::new(NetworkType::Mainnet),
            name: "jio-mainnet",
            genesis,
            ghostdag_k: DEFAULT_GHOSTDAG_K,
            target_time_per_block: DEFAULT_TARGET_TIME_PER_BLOCK_MS,
            max_block_mass: DEFAULT_MAX_BLOCK_MASS,
            finality_depth: DEFAULT_FINALITY_DEPTH,
            pruning_depth: DEFAULT_PRUNING_DEPTH,
            min_difficulty_bits: 0x1e7fffff,
            max_difficulty_target: Uint256::from_limbs([
                0x00000000ffffffff,
                0x0000000000000000,
                0x0000000000000000,
                0x0000000000000000,
            ]),
            difficulty_window_size: DEFAULT_DIFFICULTY_WINDOW_SIZE,
            coinbase_maturity: 100,
        }
    }

    pub fn testnet(suffix: Option<u32>) -> Self {
        let net = match suffix {
            Some(s) => NetworkId::with_suffix(NetworkType::Testnet, s),
            None => NetworkId::new(NetworkType::Testnet),
        };

        let genesis = GenesisBlock::build_genesis_block(
            1700000000000,
            0x1e7fffff,
            b"Jio Testnet Genesis".to_vec(),
        );

        Self {
            net,
            name: "jio-testnet",
            genesis,
            ghostdag_k: DEFAULT_GHOSTDAG_K,
            target_time_per_block: DEFAULT_TARGET_TIME_PER_BLOCK_MS,
            max_block_mass: DEFAULT_MAX_BLOCK_MASS,
            finality_depth: DEFAULT_FINALITY_DEPTH,
            pruning_depth: DEFAULT_PRUNING_DEPTH,
            min_difficulty_bits: 0x1e7fffff,
            max_difficulty_target: Uint256::from_limbs([
                0x00000000ffffffff,
                0x0000000000000000,
                0x0000000000000000,
                0x0000000000000000,
            ]),
            difficulty_window_size: DEFAULT_DIFFICULTY_WINDOW_SIZE,
            coinbase_maturity: 100,
        }
    }

    pub fn devnet() -> Self {
        let genesis = GenesisBlock::build_genesis_block(
            1700000000000,
            0x1e7fffff,
            b"Jio Devnet Genesis".to_vec(),
        );

        Self {
            net: NetworkId::new(NetworkType::Devnet),
            name: "jio-devnet",
            genesis,
            ghostdag_k: DEFAULT_GHOSTDAG_K,
            target_time_per_block: DEFAULT_TARGET_TIME_PER_BLOCK_MS,
            max_block_mass: DEFAULT_MAX_BLOCK_MASS,
            finality_depth: DEFAULT_FINALITY_DEPTH,
            pruning_depth: DEFAULT_PRUNING_DEPTH,
            min_difficulty_bits: 0x1e7fffff,
            max_difficulty_target: Uint256::from_limbs([
                0x00000000ffffffff,
                0x0000000000000000,
                0x0000000000000000,
                0x0000000000000000,
            ]),
            difficulty_window_size: DEFAULT_DIFFICULTY_WINDOW_SIZE,
            coinbase_maturity: 10,
        }
    }

    pub fn simnet() -> Self {
        let genesis = GenesisBlock::build_genesis_block(
            1700000000000,
            0x207fffff,
            b"Jio Simnet Genesis".to_vec(),
        );

        Self {
            net: NetworkId::new(NetworkType::Simnet),
            name: "jio-simnet",
            genesis,
            ghostdag_k: DEFAULT_GHOSTDAG_K,
            target_time_per_block: DEFAULT_TARGET_TIME_PER_BLOCK_MS,
            max_block_mass: DEFAULT_MAX_BLOCK_MASS,
            finality_depth: DEFAULT_FINALITY_DEPTH,
            pruning_depth: DEFAULT_PRUNING_DEPTH,
            min_difficulty_bits: 0x207fffff,
            max_difficulty_target: Uint256::from_limbs([
                0x00000000ffffffff,
                0x0000000000000000,
                0x0000000000000000,
                0x0000000000000000,
            ]),
            difficulty_window_size: DEFAULT_DIFFICULTY_WINDOW_SIZE,
            coinbase_maturity: 10,
        }
    }
}
