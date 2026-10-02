//! Trusted sync data imported during IBD.

use super::block::Block;
use jio_hashes::Hash;
use jio_math::Uint256;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrustedBlock {
    pub block: Block,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExternalGhostdagData {
    pub blue_score: u64,
    pub blue_work: Uint256,
    pub selected_parent: Hash,
    pub mergeset_blues: Vec<Hash>,
    pub mergeset_reds: Vec<Hash>,
}
