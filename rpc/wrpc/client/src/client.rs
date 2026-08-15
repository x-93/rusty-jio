use jio_rpc_core::api::rpc::{RpcApi, RpcResult};
use jio_rpc_core::model::message::*;
use async_trait::async_trait;

pub struct WrpcClient {
    pub url: String,
}

impl WrpcClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[async_trait]
impl RpcApi for WrpcClient {
    async fn ping(&self) -> RpcResult<()> {
        Ok(())
    }

    async fn get_info(&self) -> RpcResult<GetInfoResponse> {
        Ok(GetInfoResponse {
            p2p_id: "wrpc-client".to_string(),
            mempool_size: 0,
            server_version: "0.1.0".to_string(),
            is_utxo_indexed: true,
            is_synced: true,
            has_notify_command: true,
        })
    }

    async fn get_current_network(&self) -> RpcResult<GetCurrentNetworkResponse> {
        Ok(GetCurrentNetworkResponse {
            network: jio_consensus_core::network::NetworkType::Devnet,
        })
    }

    async fn submit_block(&self, _request: SubmitBlockRequest) -> RpcResult<SubmitBlockResponse> {
        Ok(SubmitBlockResponse {
            hash: jio_hashes::Hash::default(),
        })
    }

    async fn get_block_template(&self, _request: GetBlockTemplateRequest) -> RpcResult<GetBlockTemplateResponse> {
        Err(jio_rpc_core::api::rpc::RpcError::NotSupported("not supported".to_string()))
    }

    async fn submit_transaction(&self, _request: SubmitTransactionRequest) -> RpcResult<SubmitTransactionResponse> {
        Ok(SubmitTransactionResponse {
            transaction_id: jio_hashes::Hash::default(),
        })
    }

    async fn get_utxos_by_addresses(&self, _request: GetUtxosByAddressesRequest) -> RpcResult<GetUtxosByAddressesResponse> {
        Ok(GetUtxosByAddressesResponse {
            entries: std::collections::HashMap::new(),
        })
    }

    async fn get_balance_by_address(&self, _request: GetBalanceByAddressRequest) -> RpcResult<GetBalanceByAddressResponse> {
        Ok(GetBalanceByAddressResponse { balance: 0 })
    }
}
