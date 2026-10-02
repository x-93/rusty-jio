//! Pruning data structures and proofs.

use std::sync::Arc;
use jio_hashes::Hash;
use super::header::Header;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PruningPointProof {
    pub headers: Vec<Vec<Arc<Header>>>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PruningPointTrustedData {
    pub pruning_point: Arc<Header>,
    pub anticone: Vec<Hash>,
}
