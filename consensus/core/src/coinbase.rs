//! Coinbase payload builder, miner data, and block subsidy decay calculations.

use super::tx::ScriptPublicKey;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct MinerData {
    pub script_public_key: ScriptPublicKey,
    pub extra_data: Vec<u8>,
}

impl MinerData {
    pub fn new(script_public_key: ScriptPublicKey, extra_data: Vec<u8>) -> Self {
        Self { script_public_key, extra_data }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct CoinbaseData {
    pub blue_score: u64,
    pub subsidy: u64,
    pub miner_data: MinerData,
}

/// Calculates the block subsidy (in Sompi) given the DAA score.
pub fn calc_block_subsidy(daa_score: u64, _target_bps: u64) -> u64 {
    // 50 Jio Coins initial base reward = 50 * 10^8 Sompi per second
    let base_reward = 50 * 100_000_000;
    // Halving interval: every 31_536_000 DAA score (~1 year at 1 BPS)
    let halvings = daa_score / 31_536_000;
    if halvings >= 64 {
        0
    } else {
        base_reward >> halvings
    }
}
