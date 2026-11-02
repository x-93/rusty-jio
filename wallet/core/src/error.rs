use jio_bip32::Error as BIP32Error;
use jio_rpc_core::RpcError as JioRpcError;
use jio_wrpc_client::error::Error as JioWorkflowRpcError;
use workflow_rpc::client::error::Error as RpcError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error: {0}")]
    String(String),

    #[error("RPC error: {0}")]
    JioRpcClientResult(#[from] JioRpcError),

    #[error("RPC error: {0}")]
    RpcError(#[from] RpcError),

    #[error("RPC error: {0}")]
    JioWorkflowRpcError(#[from] JioWorkflowRpcError),

    #[error("BIP32 error: {0}")]
    BIP32Error(#[from] BIP32Error),
}
