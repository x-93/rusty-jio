use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GhostdagDataPackage {
    pub blue_score: u64,
    pub mergeset_blues: Vec<jio_consensus_core::blockhash::BlockHash>,
    pub mergeset_reds: Vec<jio_consensus_core::blockhash::BlockHash>,
}
