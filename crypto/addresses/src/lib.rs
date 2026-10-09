//! Jio blockchain address formatting and Bech32 encoding.

pub mod bech32;

pub use bech32::{decode_address, encode_address, Address, AddressError, Prefix, Version};
