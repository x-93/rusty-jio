use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use jio_addresses::Address;
use jio_consensus_core::tx::{ScriptPublicKey, ScriptVec, TransactionId, TransactionOutpoint, UtxoEntry};
use serde::{Deserialize, Serialize};

use crate::prelude::{RpcHash, RpcScriptClass, RpcSubnetworkId};

/// Represents the ID of a Jio transaction
pub type RpcTransactionId = TransactionId;

pub type RpcScriptVec = ScriptVec;
pub type RpcScriptPublicKey = ScriptPublicKey;
pub type RpcUtxoEntry = UtxoEntry;

/// Represents a Jio transaction outpoint
pub type RpcTransactionOutpoint = TransactionOutpoint;

/// Represents a Jio transaction input
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionInput {
    pub previous_outpoint: RpcTransactionOutpoint,
    pub signature_script: Vec<u8>,
    pub sequence: u64,
    pub sig_op_count: u8,
    pub verbose_data: Option<RpcTransactionInputVerboseData>,
}

/// Represent Jio transaction input verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionInputVerboseData {}

/// Represents a Jiopad transaction output
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionOutput {
    pub value: u64,
    pub script_public_key: RpcScriptPublicKey,
    pub verbose_data: Option<RpcTransactionOutputVerboseData>,
}

/// Represent Jio transaction output verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionOutputVerboseData {
    pub script_public_key_type: RpcScriptClass,
    pub script_public_key_address: Address,
}

/// Represents a Jio transaction
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransaction {
    pub version: u16,
    pub inputs: Vec<RpcTransactionInput>,
    pub outputs: Vec<RpcTransactionOutput>,
    pub lock_time: u64,
    pub subnetwork_id: RpcSubnetworkId,
    pub gas: u64,
    pub payload: Vec<u8>,
    pub verbose_data: Option<RpcTransactionVerboseData>,
}

/// Represent Jio transaction verbose data
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionVerboseData {
    pub transaction_id: RpcTransactionId,
    pub hash: RpcHash,
    pub mass: u64,
    pub block_hash: RpcHash,
    pub block_time: u64,
}

/// Represents accepted transaction ids
#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize, BorshSchema)]
#[serde(rename_all = "camelCase")]
pub struct RpcAcceptedTransactionIds {
    pub accepting_block_hash: RpcHash,
    pub accepted_transaction_ids: Vec<RpcTransactionId>,
}
