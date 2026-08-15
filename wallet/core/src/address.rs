use jio_addresses::Address;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletAddress {
    pub address: Address,
    pub index: u32,
    pub is_change: bool,
}
