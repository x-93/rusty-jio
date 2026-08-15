use jio_consensus_core::blockhash::BlockHash;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustedDataPackage {
    pub daa_window: Vec<BlockHash>,
    pub ghostdag_data: Vec<u8>,
}
