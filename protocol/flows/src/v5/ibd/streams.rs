use jio_consensus_core::blockhash::BlockHash;

pub struct IbdStream {
    pub current_locator: Vec<BlockHash>,
}

impl IbdStream {
    pub fn new() -> Self {
        Self {
            current_locator: Vec::new(),
        }
    }
}

impl Default for IbdStream {
    fn default() -> Self {
        Self::new()
    }
}
