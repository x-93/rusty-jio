use super::error::Result;
use core::fmt::Debug;
use jio_grpc_core::protowire::{JiopadRequest, JiopadResponse};
use jio_rpc_core::api::ops::RpcApiOps;
use std::{sync::Arc, time::Duration};
use tokio::sync::oneshot;

pub(crate) mod id;
pub(crate) mod matcher;
pub(crate) mod queue;

pub(crate) trait Resolver: Send + Sync + Debug {
    fn register_request(&self, op: RpcApiOps, request: &JiopadRequest) -> JiopadResponseReceiver;
    fn handle_response(&self, response: JiopadResponse);
    fn remove_expired_requests(&self, timeout: Duration);
}

pub(crate) type DynResolver = Arc<dyn Resolver>;

pub(crate) type JiopadResponseSender = oneshot::Sender<Result<JiopadResponse>>;
pub(crate) type JiopadResponseReceiver = oneshot::Receiver<Result<JiopadResponse>>;
