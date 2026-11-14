// tx.rs
//
// Core Transaction data models for rusty-jio.

pub mod script_public_key;
pub use script_public_key::{ScriptPublicKey, ScriptPublicKeyVersion, ScriptVec};

use crate::subnets::SubnetworkId;
use borsh::{BorshDeserialize, BorshSerialize};
use jio_hashes::{Hash, HasherBase, TransactionHash, TransactionID};
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};
use std::mem::size_of;
use std::sync::Arc;

/// Canonical transaction identifier (computed without signature scripts).
pub type TransactionId = Hash;

/// A reference to a specific output of a previous transaction.
#[derive(
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct TransactionOutpoint {
    pub transaction_id: Hash,
    pub index: u32,
}

impl TransactionOutpoint {
    #[inline(always)]
    pub fn new(transaction_id: Hash, index: u32) -> Self {
        Self {
            transaction_id,
            index,
        }
    }
}

/// An input spending an existing UTXO.
#[derive(
    Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct TransactionInput {
    pub previous_outpoint: TransactionOutpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u64,
    pub sig_op_count: u8,
}

impl TransactionInput {
    pub fn new(
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

/// A newly created output (UTXO).
#[derive(
    Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

impl TransactionOutput {
    #[inline(always)]
    pub fn new(value: u64, script_public_key: ScriptPublicKey) -> Self {
        Self {
            value,
            script_public_key,
        }
    }
}

/// A complete cryptocurrency transaction.
#[derive(
    Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct Transaction {
    pub version: u16,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: SubnetworkId,
    pub gas: u64,
    pub payload: Vec<u8>,
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
        Self {
            version,
            inputs,
            outputs,
            lock_time,
            subnetwork_id,
            gas,
            payload,
        }
    }

    /// Computes the unique Transaction ID (Hash without signature scripts).
    pub fn id(&self) -> Hash {
        let mut hasher = TransactionID::new();
        hasher.update(self.version.to_le_bytes());
        hasher.update((self.inputs.len() as u64).to_le_bytes());
        for input in &self.inputs {
            hasher.update(input.previous_outpoint.transaction_id.as_bytes());
            hasher.update(input.previous_outpoint.index.to_le_bytes());
            hasher.update(input.sequence.to_le_bytes());
        }
        hasher.update((self.outputs.len() as u64).to_le_bytes());
        for output in &self.outputs {
            hasher.update(output.value.to_le_bytes());
            hasher.update(output.script_public_key.version.to_le_bytes());
            hasher.update(output.script_public_key.script.as_slice());
        }
        hasher.update(self.lock_time.to_le_bytes());
        hasher.update(self.subnetwork_id.0);
        hasher.update(self.gas.to_le_bytes());
        hasher.update(&self.payload);
        hasher.finalize()
    }

    /// Computes the full Transaction Hash (including signature scripts).
    pub fn hash(&self) -> Hash {
        let mut hasher = TransactionHash::new();
        hasher.update(self.id().as_bytes());
        for input in &self.inputs {
            hasher.update(&input.signature_script);
        }
        hasher.finalize()
    }

    /// Checks if this is a coinbase transaction (mining reward).
    #[inline(always)]
    pub fn is_coinbase(&self) -> bool {
        self.subnetwork_id.is_coinbase()
    }
}

/// Shared immutable transaction reference used throughout consensus pipelines.
pub type ArcTransaction = Arc<Transaction>;

/// UTXO entry representing an unspent transaction output in the DAG.
#[derive(
    Clone, Default, PartialEq, Eq, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct UtxoEntry {
    pub amount: u64,
    pub script_public_key: ScriptPublicKey,
    pub block_daa_score: u64,
    pub is_coinbase: bool,
}

impl UtxoEntry {
    pub fn new(
        amount: u64,
        script_public_key: ScriptPublicKey,
        block_daa_score: u64,
        is_coinbase: bool,
    ) -> Self {
        Self {
            amount,
            script_public_key,
            block_daa_score,
            is_coinbase,
        }
    }
}

impl MemSizeEstimator for TransactionOutpoint {
    #[inline(always)]
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
    }
}

impl MemSizeEstimator for TransactionInput {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.signature_script.capacity()
    }
}

impl MemSizeEstimator for TransactionOutput {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.script_public_key.script.len()
    }
}

impl MemSizeEstimator for Transaction {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
            + self
                .inputs
                .iter()
                .map(TransactionInput::estimate_mem_bytes)
                .sum::<usize>()
            + self
                .outputs
                .iter()
                .map(TransactionOutput::estimate_mem_bytes)
                .sum::<usize>()
            + self.payload.capacity()
    }
}

impl MemSizeEstimator for UtxoEntry {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.script_public_key.estimate_mem_bytes()
    }
}
