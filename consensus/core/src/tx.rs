pub mod script_public_key;

use borsh::{BorshDeserialize, BorshSerialize};
use jio_utils::mem_size::MemSizeEstimator;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

use crate::subnets::{SubnetworkId, SUBNETWORK_ID_COINBASE};
pub use script_public_key::*;

pub use jio_hashes::Hash as TransactionId;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOutpoint {
    pub transaction_id: TransactionId,
    pub index: u32,
}

impl TransactionOutpoint {
    pub const fn new(transaction_id: TransactionId, index: u32) -> Self {
        Self { transaction_id, index }
    }
}

impl Debug for TransactionOutpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.transaction_id, self.index)
    }
}

impl Display for TransactionOutpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.transaction_id, self.index)
    }
}

impl MemSizeEstimator for TransactionOutpoint {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>()
    }
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
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

impl Debug for TransactionInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionInput")
            .field("previous_outpoint", &self.previous_outpoint)
            .field("signature_script_len", &self.signature_script.len())
            .field("sequence", &self.sequence)
            .field("sig_op_count", &self.sig_op_count)
            .finish()
    }
}

impl MemSizeEstimator for TransactionInput {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.signature_script.capacity()
    }
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionOutput {
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

impl TransactionOutput {
    pub fn new(value: u64, script_public_key: ScriptPublicKey) -> Self {
        Self {
            value,
            script_public_key,
        }
    }
}

impl Debug for TransactionOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionOutput")
            .field("value", &self.value)
            .field("script_public_key", &self.script_public_key)
            .finish()
    }
}

impl MemSizeEstimator for TransactionOutput {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.script_public_key.estimate_mem_bytes()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(rename_all = "camelCase")]
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

impl Debug for UtxoEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UtxoEntry")
            .field("amount", &self.amount)
            .field("script_public_key", &self.script_public_key)
            .field("block_daa_score", &self.block_daa_score)
            .field("is_coinbase", &self.is_coinbase)
            .finish()
    }
}

impl MemSizeEstimator for UtxoEntry {
    fn estimate_mem_bytes(&self) -> usize {
        size_of::<Self>() + self.script_public_key.estimate_mem_bytes()
    }
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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
            mass: 0,
        }
    }

    pub fn id(&self) -> TransactionId {
        crate::hashing::tx::id(self)
    }

    pub fn is_coinbase(&self) -> bool {
        self.subnetwork_id == SUBNETWORK_ID_COINBASE
    }
}

impl Debug for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transaction")
            .field("version", &self.version)
            .field("inputs_len", &self.inputs.len())
            .field("outputs_len", &self.outputs.len())
            .field("lock_time", &self.lock_time)
            .field("subnetwork_id", &self.subnetwork_id)
            .field("payload_len", &self.payload.len())
            .field("mass", &self.mass)
            .finish()
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

pub trait VerifiableTransaction {
    fn id(&self) -> TransactionId;
    fn populated_inputs<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a TransactionInput, &'a UtxoEntry)> + 'a>;
    fn outputs(&self) -> &[TransactionOutput];
    fn is_coinbase(&self) -> bool;
}

pub struct PopulatedTransaction<'a> {
    pub tx: &'a Transaction,
    pub entries: Vec<UtxoEntry>,
}

impl<'a> PopulatedTransaction<'a> {
    pub fn new(tx: &'a Transaction, entries: Vec<UtxoEntry>) -> Self {
        assert_eq!(tx.inputs.len(), entries.len());
        Self { tx, entries }
    }
}

impl<'a> VerifiableTransaction for PopulatedTransaction<'a> {
    fn id(&self) -> TransactionId {
        self.tx.id()
    }

    fn populated_inputs<'b>(&'b self) -> Box<dyn Iterator<Item = (&'b TransactionInput, &'b UtxoEntry)> + 'b> {
        Box::new(self.tx.inputs.iter().zip(self.entries.iter()))
    }

    fn outputs(&self) -> &[TransactionOutput] {
        &self.tx.outputs
    }

    fn is_coinbase(&self) -> bool {
        self.tx.is_coinbase()
    }
}
