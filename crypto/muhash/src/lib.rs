//! Multi-set associative rolling hash implementation for UTXO set commitment updates.

pub mod u3072;

pub use u3072::{MuHash, Num3072, BYTES, C, LIMBS};
