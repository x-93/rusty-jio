use jio_rpc_service::RpcCoreService;
use std::sync::Arc;

pub struct WrpcService {
    pub core_service: Arc<RpcCoreService>,
}

impl WrpcService {
    pub fn new(core_service: Arc<RpcCoreService>) -> Self {
        Self { core_service }
    }
}
