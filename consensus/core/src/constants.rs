pub const MAX_BLOCK_PARENTS: usize = 16;

/// Number of sompi (smallest divisible unit) per 1 Jio coin.
pub const SOMPI_PER_JIO: u64 = 100_000_000;

/// Maximum total circulating Sompi supply.
pub const MAX_SOMPI: u64 = 28_700_000_000 * SOMPI_PER_JIO;

/// The standard block header version.
pub const BLOCK_VERSION: u16 = 1;

/// The standard transaction version.
pub const TX_VERSION: u16 = 0;

/// Special sentinel value for unaccepted DAA score.
pub const UNACCEPTED_DAA_SCORE: u64 = u64::MAX;

/// Maximum transaction input sequence number.
pub const MAX_TX_IN_SEQUENCE_NUM: u64 = u64::MAX;

/// Maximum allowed script public key size in bytes.
pub const MAX_SCRIPT_PUBLIC_KEY_SIZE: usize = 1024;

/// Default PoW difficulty target compact representation (bits).
pub const DEFAULT_MIN_DIFFICULTY_BITS: u32 = 0x1e7fffff;
