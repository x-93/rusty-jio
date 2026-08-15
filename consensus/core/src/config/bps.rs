use crate::KType;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bps<const B: u64>;

impl<const B: u64> Bps<B> {
    pub const fn bps() -> u64 {
        B
    }

    pub const fn past_median_time_sample_rate() -> u64 {
        B
    }

    pub const fn difficulty_adjustment_sample_rate() -> u64 {
        B
    }

    pub const fn target_time_per_block() -> u64 {
        1000 / B
    }
}

pub struct Testnet11Bps;

impl Testnet11Bps {
    pub const fn bps() -> u64 {
        10
    }

    pub const fn ghostdag_k() -> KType {
        18
    }

    pub const fn target_time_per_block() -> u64 {
        100
    }

    pub const fn past_median_time_sample_rate() -> u64 {
        10
    }

    pub const fn difficulty_adjustment_sample_rate() -> u64 {
        10
    }

    pub const fn max_block_parents() -> u8 {
        10
    }

    pub const fn mergeset_size_limit() -> u64 {
        180
    }

    pub const fn merge_depth_bound() -> u64 {
        3600
    }

    pub const fn finality_depth() -> u64 {
        86400
    }

    pub const fn pruning_depth() -> u64 {
        185798
    }

    pub const fn pruning_proof_m() -> u64 {
        1000
    }

    pub const fn deflationary_phase_daa_score() -> u64 {
        15778800 - 259200
    }

    pub const fn pre_deflationary_phase_base_subsidy() -> u64 {
        50000000000
    }

    pub const fn coinbase_maturity() -> u64 {
        100
    }
}

