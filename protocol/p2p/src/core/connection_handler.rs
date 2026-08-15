use crate::core::hub::Hub;
use crate::core::peer::Peer;
use crate::core::router::Router;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::channel;

pub struct ConnectionHandler {
    hub: Hub,
}

impl ConnectionHandler {
    pub fn new(hub: Hub) -> Self {
        Self { hub }
    }

    pub fn handle_new_connection(
        &self,
        addr: SocketAddr,
        is_outbound: bool,
    ) -> (Arc<Peer>, tokio::sync::mpsc::Receiver<Vec<u8>>) {
        let (tx, rx) = channel(256);
        let router = Arc::new(Router::new(tx));
        let peer = Arc::new(Peer::new(addr, router, is_outbound));
        self.hub.register(peer.clone());
        (peer, rx)
    }

    pub fn handle_disconnection(&self, addr: &SocketAddr) {
        self.hub.unregister(addr);
    }
}
