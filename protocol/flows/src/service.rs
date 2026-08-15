use crate::flow_context::FlowContext;
use crate::flow_trait::Flow;
use crate::v5::ping::PingFlow;
use jio_p2p::core::peer::Peer;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct P2pService {
    ctx: Arc<FlowContext>,
}

impl P2pService {
    pub fn new(ctx: Arc<FlowContext>) -> Self {
        Self { ctx }
    }

    pub async fn start(&self, listen_addr: SocketAddr) -> Result<(), String> {
        match tokio::net::TcpListener::bind(listen_addr).await {
            Ok(listener) => {
                log::info!("P2P listening on {}", listen_addr);
                let ctx = self.ctx.clone();
                tokio::spawn(async move {
                    loop {
                        if let Ok((_stream, peer_addr)) = listener.accept().await {
                            log::info!("P2P connection accepted from {}", peer_addr);
                            let (tx, _rx) = tokio::sync::mpsc::channel(128);
                            let router = Arc::new(jio_p2p::core::router::Router::new(tx));
                            let peer = Arc::new(Peer::new(peer_addr, router, false));
                            let mut ping_flow = PingFlow::new(ctx.clone(), peer.clone());
                            tokio::spawn(async move {
                                let _ = ping_flow.start().await;
                            });
                        }
                    }
                });
            }
            Err(e) => {
                log::warn!("Could not bind P2P listener on {}: {}", listen_addr, e);
            }
        }

        Ok(())
    }

    pub fn start_peer_flows(&self, peer: Arc<Peer>) {
        let ctx = self.ctx.clone();
        let p = peer.clone();
        tokio::spawn(async move {
            let mut ping_flow = PingFlow::new(ctx, p);
            let _ = ping_flow.start().await;
        });
    }
}
