use jio_consensus_core::blockhash::BlockHash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruningPointData {
    pub pruning_point: BlockHash,
    pub anticone: Vec<BlockHash>,
}
