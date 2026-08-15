use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountBalance {
    pub mature: u64,
    pub pending: u64,
    pub outgoing: u64,
}

impl AccountBalance {
    pub fn total(&self) -> u64 {
        self.mature.saturating_add(self.pending)
    }
}
