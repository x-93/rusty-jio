pub mod script_public_key;

use crate::hashing;
use crate::subnets::SubnetworkId;
use borsh::{BorshDeserialize, BorshSerialize};
pub use script_public_key::{ScriptPublicKey, ScriptVec};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

pub type TransactionId = jio_hashes::Hash;
pub type TransactionArc = Arc<Transaction>;

/// Reference to a specific transaction output in the UTXO set.
#[derive(
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOutpoint {
    pub transaction_id: TransactionId,
    pub index: u32,
}

impl TransactionOutpoint {
    pub const fn new(transaction_id: TransactionId, index: u32) -> Self {
        Self {
            transaction_id,
            index,
        }
    }
}

impl Display for TransactionOutpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.transaction_id, self.index)
    }
}

impl Debug for TransactionOutpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl FromStr for TransactionOutpoint {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tx_id_str, index_str) = s.split_once(':').ok_or("expected format <txid>:<index>")?;
        let transaction_id =
            TransactionId::from_str(tx_id_str).map_err(|_| "invalid transaction id hex")?;
        let index = index_str
            .parse::<u32>()
            .map_err(|_| "invalid outpoint index")?;
        Ok(Self {
            transaction_id,
            index,
        })
    }
}

/// A transaction input referencing a prior output in the UTXO set.
#[derive(
    Clone, Default, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInput {
    pub previous_outpoint: TransactionOutpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u64,
    pub sig_op_count: u8,
}

impl TransactionInput {
    pub const fn new(
        previous_outpoint: TransactionOutpoint,
        signature_script: Vec<u8>,
        sequence: u64,
        sig_op_count: u8,
    ) -> Self {
        Self {
            previous_outpoint,
            signature_script,
            sequence,
            sig_op_count,
        }
    }
}

/// A transaction output transferring value and locking under a ScriptPublicKey.
#[derive(
    Clone, Default, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

impl TransactionOutput {
    pub const fn new(value: u64, script_public_key: ScriptPublicKey) -> Self {
        Self {
            value,
            script_public_key,
        }
    }
}

/// A full transaction in the Jio consensus layer.
#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub version: u16,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    pub payload: Vec<u8>,
    pub mass: u64,
    pub id: TransactionId,
}

impl Transaction {
    pub fn new(
        version: u16,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u64,
        subnetwork_id: SubnetworkId,
        gas: u64,
        payload: Vec<u8>,
    ) -> Self {
        let mut tx = Self {
            version,
            inputs,
            outputs,
            lock_time,
            subnetwork_id,
            gas,
            payload,
            mass: 0,
            id: Default::default(),
        };
        tx.finalize();
        tx
    }

    /// Computes and assigns the cached non-malleable TransactionId.
    pub fn finalize(&mut self) {
        self.id = hashing::tx::id(self);
    }

    /// Returns the unique, non-malleable transaction ID.
    #[inline(always)]
    pub fn id(&self) -> TransactionId {
        self.id
    }

    /// Returns the full witness hash of the transaction.
    #[inline(always)]
    pub fn hash(&self) -> jio_hashes::Hash {
        hashing::tx::hash(self)
    }

    /// Returns true if this is a coinbase transaction.
    #[inline(always)]
    pub fn is_coinbase(&self) -> bool {
        self.subnetwork_id.is_coinbase()
    }

    /// Returns true if this is a native standard transaction.
    #[inline(always)]
    pub fn is_native(&self) -> bool {
        self.subnetwork_id.is_native()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subnets::SUBNETWORK_ID_NATIVE;

    #[test]
    fn test_transaction_id_malleability_resistance() {
        let outpoint = TransactionOutpoint::new(TransactionId::from([1u8; 32]), 0);
        let input = TransactionInput::new(outpoint, vec![0x11, 0x22], 0, 1);
        let output = TransactionOutput::new(100_000_000, ScriptPublicKey::new(0, vec![0x51]));

        let tx1 = Transaction::new(
            1,
            vec![input],
            vec![output.clone()],
            0,
            SUBNETWORK_ID_NATIVE,
            0,
            vec![],
        );

        // Same tx with modified signature script
        let modified_input = TransactionInput::new(outpoint, vec![0x99, 0xaa, 0xbb], 0, 1);
        let tx2 = Transaction::new(
            1,
            vec![modified_input],
            vec![output],
            0,
            SUBNETWORK_ID_NATIVE,
            0,
            vec![],
        );

        // TransactionId MUST remain strictly identical (malleability protection)
        assert_eq!(tx1.id(), tx2.id());

        // Full TransactionHash MUST differ due to distinct witnesses
        assert_ne!(tx1.hash(), tx2.hash());
    }

    #[test]
    fn test_outpoint_string_roundtrip() {
        let op = TransactionOutpoint::new(TransactionId::from([0x42u8; 32]), 3);
        let s = op.to_string();
        let parsed = TransactionOutpoint::from_str(&s).unwrap();
        assert_eq!(op, parsed);
    }
}
