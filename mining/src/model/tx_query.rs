use jio_consensus_core::tx::TransactionId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionQueryResponse {
    pub tx_id: TransactionId,
    pub fee: u64,
    pub mass: u64,
    pub is_orphan: bool,
}
