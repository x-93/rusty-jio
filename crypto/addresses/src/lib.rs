//! Jio blockchain address formatting and Bech32 encoding.

pub mod bech32;

pub use bech32::{Address, AddressError, Prefix, Version, decode_address, encode_address};
