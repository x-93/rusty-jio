//! AcceptanceData recording transactions merged and accepted per block.

use super::tx::TransactionId;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct AcceptanceData {
    pub accepted_transactions: Vec<TransactionId>,
}
