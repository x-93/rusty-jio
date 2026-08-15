use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockStatus {
    StatusInvalid = 0,
    StatusHeaderOnly = 1,
    StatusUTXOValid = 2,
    StatusUTXOInvalid = 3,
    StatusDisqualifiedFromChain = 4,
}

impl BlockStatus {
    pub fn is_valid(&self) -> bool {
        !matches!(self, BlockStatus::StatusInvalid | BlockStatus::StatusUTXOInvalid)
    }

    pub fn is_header_only(&self) -> bool {
        matches!(self, BlockStatus::StatusHeaderOnly)
    }

    pub fn is_utxo_valid(&self) -> bool {
        matches!(self, BlockStatus::StatusUTXOValid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_status_predicates() {
        assert!(!BlockStatus::StatusInvalid.is_valid());
        assert!(!BlockStatus::StatusInvalid.is_header_only());
        assert!(!BlockStatus::StatusInvalid.is_utxo_valid());

        assert!(BlockStatus::StatusHeaderOnly.is_valid());
        assert!(BlockStatus::StatusHeaderOnly.is_header_only());
        assert!(!BlockStatus::StatusHeaderOnly.is_utxo_valid());

        assert!(BlockStatus::StatusUTXOValid.is_valid());
        assert!(!BlockStatus::StatusUTXOValid.is_header_only());
        assert!(BlockStatus::StatusUTXOValid.is_utxo_valid());

        assert!(!BlockStatus::StatusUTXOInvalid.is_valid());
        assert!(!BlockStatus::StatusUTXOInvalid.is_header_only());
        assert!(!BlockStatus::StatusUTXOInvalid.is_utxo_valid());

        assert!(BlockStatus::StatusDisqualifiedFromChain.is_valid());
        assert!(!BlockStatus::StatusDisqualifiedFromChain.is_header_only());
        assert!(!BlockStatus::StatusDisqualifiedFromChain.is_utxo_valid());
    }

    #[test]
    fn test_block_status_serde_roundtrip() {
        let statuses = [
            BlockStatus::StatusInvalid,
            BlockStatus::StatusHeaderOnly,
            BlockStatus::StatusUTXOValid,
            BlockStatus::StatusUTXOInvalid,
            BlockStatus::StatusDisqualifiedFromChain,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: BlockStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}

