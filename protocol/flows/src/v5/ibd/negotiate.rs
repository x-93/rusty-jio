use jio_consensus_core::blockhash::BlockHash;

pub struct IbdNegotiation {
    pub highest_known_hash: Option<BlockHash>,
}

impl IbdNegotiation {
    pub fn new() -> Self {
        Self {
            highest_known_hash: None,
        }
    }
}

impl Default for IbdNegotiation {
    fn default() -> Self {
        Self::new()
    }
}
