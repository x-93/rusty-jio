use crate::subnets::SUBNETWORK_ID_COINBASE;
use crate::tx::{ScriptPublicKey, Transaction, TransactionOutput};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Decoded payload stored inside a Coinbase transaction.
#[derive(
    Clone, Debug, PartialEq, Eq, Default, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct CoinbaseData {
    pub blue_score: u64,
    pub subsidy: u64,
    pub miner_data: Vec<u8>,
}

impl CoinbaseData {
    pub fn new(blue_score: u64, subsidy: u64, miner_data: Vec<u8>) -> Self {
        Self {
            blue_score,
            subsidy,
            miner_data,
        }
    }

    /// Serializes coinbase fields into raw payload bytes.
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(16 + self.miner_data.len());
        payload.extend_from_slice(&self.blue_score.to_le_bytes());
        payload.extend_from_slice(&self.subsidy.to_le_bytes());
        payload.extend_from_slice(&self.miner_data);
        payload
    }

    /// Deserializes coinbase data from raw transaction payload bytes.
    pub fn deserialize(payload: &[u8]) -> Result<Self, &'static str> {
        if payload.len() < 16 {
            return Err("coinbase payload too short, expected at least 16 bytes");
        }
        let blue_score = u64::from_le_bytes(payload[0..8].try_into().unwrap());
        let subsidy = u64::from_le_bytes(payload[8..16].try_into().unwrap());
        let miner_data = payload[16..].to_vec();
        Ok(Self {
            blue_score,
            subsidy,
            miner_data,
        })
    }
}

/// Constructs a canonical Coinbase transaction for a mined block.
pub fn create_coinbase_transaction(
    blue_score: u64,
    subsidy: u64,
    script_public_key: ScriptPublicKey,
    miner_data: Vec<u8>,
) -> Transaction {
    let coinbase_data = CoinbaseData::new(blue_score, subsidy, miner_data);
    let payload = coinbase_data.serialize();
    let output = TransactionOutput::new(subsidy, script_public_key);

    Transaction::new(
        0,
        vec![],
        vec![output],
        0,
        SUBNETWORK_ID_COINBASE,
        0,
        payload,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coinbase_data_roundtrip() {
        let data = CoinbaseData::new(123456, 500_000_000, vec![0xaa, 0xbb, 0xcc]);
        let payload = data.serialize();
        let decoded = CoinbaseData::deserialize(&payload).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_coinbase_transaction_creation() {
        let spk = ScriptPublicKey::new(0, vec![0x51]);
        let tx = create_coinbase_transaction(100, 50_000_000, spk.clone(), vec![]);
        assert!(tx.is_coinbase());
        assert!(!tx.is_native());
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].value, 50_000_000);
        assert_eq!(tx.outputs[0].script_public_key, spk);
    }
}
