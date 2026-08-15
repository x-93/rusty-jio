use crate::model::block::RpcBlock;
use crate::model::hash::RpcHash;
use crate::model::tx::{RpcTransaction, RpcTransactionId, RpcTransactionOutpoint, RpcUtxoEntry};
use jio_consensus_core::network::NetworkType;
use jio_consensus_core::tx::ScriptPublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetInfoResponse {
    pub p2p_id: String,
    pub mempool_size: u64,
    pub server_version: String,
    pub is_utxo_indexed: bool,
    pub is_synced: bool,
    pub has_notify_command: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetCurrentNetworkResponse {
    pub network: NetworkType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitBlockRequest {
    pub block: RpcBlock,
    pub allow_non_daa_blocks: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitBlockResponse {
    pub hash: RpcHash,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetBlockTemplateRequest {
    pub payee_script_public_key: ScriptPublicKey,
    pub extra_data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetBlockTemplateResponse {
    pub block: RpcBlock,
    pub is_synced: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitTransactionRequest {
    pub transaction: RpcTransaction,
    pub allow_orphan: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitTransactionResponse {
    pub transaction_id: RpcTransactionId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetUtxosByAddressesRequest {
    pub addresses: Vec<ScriptPublicKey>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetUtxosByAddressesResponse {
    pub entries: HashMap<RpcTransactionOutpoint, RpcUtxoEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetBalanceByAddressRequest {
    pub address: ScriptPublicKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetBalanceByAddressResponse {
    pub balance: u64,
}
