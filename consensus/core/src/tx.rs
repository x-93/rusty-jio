//! Transaction, inputs, outputs, outpoints, and UTXO entry definitions.

pub mod script_public_key;
pub use script_public_key::{ScriptPublicKey, ScriptVec};

use jio_hashes::{Hash, HasherBase, TransactionHash, TransactionID};
use super::subnets::SubnetworkId;

pub type TransactionId = Hash;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct TransactionOutpoint {
    pub transaction_id: TransactionId,
    pub index: u32,
}

impl TransactionOutpoint {
    pub const fn new(transaction_id: TransactionId, index: u32) -> Self {
        Self { transaction_id, index }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct UtxoEntry {
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
    pub block_daa_score: u64,
    pub is_coinbase: bool,
}

impl UtxoEntry {
    pub fn new(amount: u64, script_public_key: ScriptPublicKey, block_daa_score: u64, is_coinbase: bool) -> Self {
        Self {
            amount,
            script_public_key,
            block_daa_score,
            is_coinbase,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct TxInput {
    pub previous_outpoint: TransactionOutpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u64,
    pub sig_op_count: u8,
}

impl TxInput {
    pub fn new(previous_outpoint: TransactionOutpoint, signature_script: Vec<u8>, sequence: u64, sig_op_count: u8) -> Self {
        Self {
            previous_outpoint,
            signature_script,
            sequence,
            sig_op_count,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct TxOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

impl TxOutput {
    pub fn new(value: u64, script_public_key: ScriptPublicKey) -> Self {
        Self { value, script_public_key }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
pub struct Transaction {
    pub version: u16,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    pub payload: Vec<u8>,
    pub mass: u64,
}

impl Transaction {
    pub fn new(
        version: u16,
        inputs: Vec<TxInput>,
        outputs: Vec<TxOutput>,
        lock_time: u64,
        subnetwork_id: SubnetworkId,
        gas: u64,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            version,
            inputs,
            outputs,
            lock_time,
            subnetwork_id,
            gas,
            payload,
            mass: 0,
        }
    }

    /// Computes the unique Transaction ID (hash of tx excluding signature scripts).
    pub fn id(&self) -> TransactionId {
        let mut hasher = TransactionID::new();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&(self.inputs.len() as u64).to_le_bytes());
        for input in &self.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(&input.previous_outpoint.index.to_le_bytes());
            hasher.update(&input.sequence.to_le_bytes());
        }
        hasher.update(&(self.outputs.len() as u64).to_le_bytes());
        for output in &self.outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.script_public_key.version.to_le_bytes());
            hasher.update(output.script_public_key.script.as_slice());
        }
        hasher.update(&self.lock_time.to_le_bytes());
        hasher.update(&self.subnetwork_id.0);
        hasher.update(&self.gas.to_le_bytes());
        hasher.update(&self.payload);
        hasher.finalize()
    }

    /// Computes full Transaction Hash (including signatures).
    pub fn hash(&self) -> Hash {
        let mut hasher = TransactionHash::new();
        hasher.update(self.id().as_bytes());
        for input in &self.inputs {
            hasher.update(&input.signature_script);
        }
        hasher.finalize()
    }
}

pub type TransactionInput = TxInput;
pub type TransactionOutput = TxOutput;
pub type MutableTransaction = Transaction;
