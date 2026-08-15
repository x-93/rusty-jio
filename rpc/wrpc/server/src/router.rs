use jio_rpc_core::api::ops::RpcApiOps;

pub struct WrpcRouter;

impl WrpcRouter {
    pub fn parse_op(op_name: &str) -> Option<RpcApiOps> {
        match op_name {
            "ping" => Some(RpcApiOps::Ping),
            "getInfo" => Some(RpcApiOps::GetInfo),
            "submitBlock" => Some(RpcApiOps::SubmitBlock),
            "getBlockTemplate" => Some(RpcApiOps::GetBlockTemplate),
            "submitTransaction" => Some(RpcApiOps::SubmitTransaction),
            "getUtxosByAddresses" => Some(RpcApiOps::GetUtxosByAddresses),
            "getBalanceByAddress" => Some(RpcApiOps::GetBalanceByAddress),
            _ => None,
        }
    }
}
