use crate::StatusResult;
use jio_core::trace;
use jio_grpc_core::protowire::JiopadResponse;
use jio_notify::{connection::Connection, error::Error as NotificationError, listener::ListenerId, notifier::Notifier};
use jio_rpc_core::Notification;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::Sender;

pub type GrpcSender = Sender<StatusResult<JiopadResponse>>;

// TODO: identify a connection by a Uuid instead of an address
// TODO: add a shutdown signal sender
#[derive(Debug)]
struct Inner {
    pub address: SocketAddr, // TODO: wrap into an option
    pub sender: GrpcSender,
    pub closed: AtomicBool,
}

#[derive(Clone, Debug)]
pub struct GrpcConnection {
    inner: Arc<Inner>,
}

impl GrpcConnection {
    pub fn new(address: SocketAddr, sender: GrpcSender) -> Self {
        Self { inner: Arc::new(Inner { address, sender, closed: AtomicBool::new(false) }) }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Default)]
pub enum GrpcEncoding {
    #[default]
    ProtowireResponse = 0,
}

impl Connection for GrpcConnection {
    type Notification = Notification;
    type Message = Arc<StatusResult<JiopadResponse>>;
    type Encoding = GrpcEncoding;
    type Error = super::error::Error;

    fn encoding(&self) -> Self::Encoding {
        GrpcEncoding::ProtowireResponse
    }

    fn into_message(notification: &jio_rpc_core::Notification, _: &Self::Encoding) -> Self::Message {
        Arc::new(Ok((notification).into()))
    }

    fn send(&self, message: Self::Message) -> Result<(), Self::Error> {
        match !self.is_closed() {
            true => Ok(self.inner.sender.try_send((*message).clone())?),
            false => Err(NotificationError::ConnectionClosed.into()),
        }
    }

    fn close(&self) -> bool {
        // TODO: actually close sender
        self.inner.closed.store(true, Ordering::SeqCst);
        true
    }

    fn is_closed(&self) -> bool {
        self.inner.sender.is_closed() || self.inner.closed.load(Ordering::SeqCst)
    }
}

pub(crate) struct GrpcConnectionManager {
    connections: HashMap<SocketAddr, GrpcConnection>,
    notifier: Arc<Notifier<Notification, GrpcConnection>>,
}

impl GrpcConnectionManager {
    pub fn new(notifier: Arc<Notifier<Notification, GrpcConnection>>) -> Self {
        Self { connections: HashMap::new(), notifier }
    }

    pub fn register(&mut self, address: SocketAddr, sender: GrpcSender) -> ListenerId {
        let connection = GrpcConnection::new(address, sender);
        let id = self.notifier.clone().register_new_listener(connection.clone());
        trace!("registering a new gRPC connection from: {address} with listener id {id}");

        // A pre-existing connection with same address is ignored here
        // TODO: see if some close pattern can be applied to the replaced connection
        self.connections.insert(address, connection);
        id
    }

    pub fn unregister(&mut self, address: SocketAddr) {
        if let Some(connection) = self.connections.remove(&address) {
            trace!("dismiss a gRPC connection from: {}", connection.inner.address);
            connection.close();
        }
    }
}
