//! Block hash newtypes and collections.

use jio_hashes::Hash;

pub type BlockHashSet = crate::BlockHashSet;

pub const ORIGIN: Hash = Hash::from_bytes([0u8; 32]);

pub trait BlockHashExtensions {
    fn is_origin(&self) -> bool;
}

impl BlockHashExtensions for Hash {
    fn is_origin(&self) -> bool {
        *self == ORIGIN
    }
}
