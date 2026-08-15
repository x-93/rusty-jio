use crate::core::connection_handler::ConnectionHandler;
use crate::core::hub::Hub;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct Adaptor {
    pub hub: Hub,
    pub connection_handler: ConnectionHandler,
    pub listen_addr: Option<SocketAddr>,
}

impl Adaptor {
    pub fn new(listen_addr: Option<SocketAddr>) -> Arc<Self> {
        let hub = Hub::new();
        let connection_handler = ConnectionHandler::new(hub.clone());
        Arc::new(Self {
            hub,
            connection_handler,
            listen_addr,
        })
    }
}
