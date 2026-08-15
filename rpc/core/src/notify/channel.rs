use crate::api::notifications::RpcNotification;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub type RpcChannel = (Sender<RpcNotification>, Receiver<RpcNotification>);

pub fn create_rpc_channel(capacity: usize) -> RpcChannel {
    channel(capacity)
}
