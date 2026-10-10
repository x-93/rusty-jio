use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockStatus {
    StatusInvalid = 0,
    StatusHeaderOnly = 1,
    StatusUTXOValid = 2,
    StatusDisqualifiedFromChain = 3,
}

impl BlockStatus {
    pub fn is_header_only(&self) -> bool {
        matches!(self, BlockStatus::StatusHeaderOnly)
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, BlockStatus::StatusUTXOValid)
    }

    pub fn has_block_body(&self) -> bool {
        !self.is_header_only()
    }
}