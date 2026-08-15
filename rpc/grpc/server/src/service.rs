use jio_rpc_service::RpcCoreService;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct GrpcServer {
    pub core_service: Arc<RpcCoreService>,
    pub listen_addr: SocketAddr,
}

impl GrpcServer {
    pub fn new(core_service: Arc<RpcCoreService>, listen_addr: SocketAddr) -> Self {
        Self {
            core_service,
            listen_addr,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        match tokio::net::TcpListener::bind(self.listen_addr).await {
            Ok(listener) => {
                log::info!("gRPC server listening on {}", self.listen_addr);
                let _core = self.core_service.clone();
                tokio::spawn(async move {
                    loop {
                        if let Ok((_stream, peer_addr)) = listener.accept().await {
                            log::debug!("RPC client connection established from {}", peer_addr);
                        }
                    }
                });
            }
            Err(e) => {
                log::warn!("Could not bind gRPC server on {}: {}", self.listen_addr, e);
            }
        }

        Ok(())
    }
}
