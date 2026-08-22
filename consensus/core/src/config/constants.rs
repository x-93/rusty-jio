use crate::KType;

/// Standard default GHOSTDAG K parameter.
pub const DEFAULT_GHOSTDAG_K: KType = 18;

/// Default block time in milliseconds (1000ms = 1 BPS).
pub const DEFAULT_TARGET_TIME_PER_BLOCK_MS: u64 = 1000;

/// Default maximum allowable mass per block.
pub const DEFAULT_MAX_BLOCK_MASS: u64 = 500_000;

/// Default finality depth in blocks.
pub const DEFAULT_FINALITY_DEPTH: u64 = 86_400;

/// Default pruning depth in blocks.
pub const DEFAULT_PRUNING_DEPTH: u64 = 185_798;

/// Sliding window size for Difficulty Adjustment Algorithm (DAA).
pub const DEFAULT_DIFFICULTY_WINDOW_SIZE: usize = 2641;
