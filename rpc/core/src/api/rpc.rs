use crate::model::message::*;
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RpcError {
    #[error("general rpc error: {0}")]
    General(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid parameter: {0}")]
    InvalidParameter(String),
    #[error("unsupported operation: {0}")]
    NotSupported(String),
}

pub type RpcResult<T> = Result<T, RpcError>;

#[async_trait]
pub trait RpcApi: Send + Sync {
    async fn ping(&self) -> RpcResult<()>;
    async fn get_info(&self) -> RpcResult<GetInfoResponse>;
    async fn get_current_network(&self) -> RpcResult<GetCurrentNetworkResponse>;
    async fn submit_block(&self, request: SubmitBlockRequest) -> RpcResult<SubmitBlockResponse>;
    async fn get_block_template(&self, request: GetBlockTemplateRequest) -> RpcResult<GetBlockTemplateResponse>;
    async fn submit_transaction(&self, request: SubmitTransactionRequest) -> RpcResult<SubmitTransactionResponse>;
    async fn get_utxos_by_addresses(&self, request: GetUtxosByAddressesRequest) -> RpcResult<GetUtxosByAddressesResponse>;
    async fn get_balance_by_address(&self, request: GetBalanceByAddressRequest) -> RpcResult<GetBalanceByAddressResponse>;
}
