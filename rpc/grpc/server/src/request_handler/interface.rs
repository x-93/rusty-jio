use super::method::{DropFn, Method, MethodTrait, RoutingPolicy};
use crate::{
    connection::Connection,
    connection_handler::ServerContext,
    error::{GrpcServerError, GrpcServerResult},
};
use jio_grpc_core::{
    ops::JiopadPayloadOps,
    protowire::{JiopadRequest, JiopadResponse},
};
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

pub type JiopadMethod = Method<ServerContext, Connection, JiopadRequest, JiopadResponse>;
pub type DynJiopadMethod = Arc<dyn MethodTrait<ServerContext, Connection, JiopadRequest, JiopadResponse>>;
pub type JiopadDropFn = DropFn<JiopadRequest, JiopadResponse>;
pub type JiopadRoutingPolicy = RoutingPolicy<JiopadRequest, JiopadResponse>;

/// An interface providing methods implementations and a fallback "not implemented" method
/// actually returning a message with a "not implemented" error.
///
/// The interface can provide a method clone for every [`JiopadPayloadOps`] variant for later
/// processing of related requests.
///
/// It is also possible to directly let the interface itself process a request by invoking
/// the `call()` method.
pub struct Interface {
    server_ctx: ServerContext,
    methods: HashMap<JiopadPayloadOps, DynJiopadMethod>,
    method_not_implemented: DynJiopadMethod,
}

impl Interface {
    pub fn new(server_ctx: ServerContext) -> Self {
        let method_not_implemented = Arc::new(Method::new(|_, _, jiopad_request: JiopadRequest| {
            Box::pin(async move {
                match jiopad_request.payload {
                    Some(ref request) => Ok(JiopadResponse {
                        id: jiopad_request.id,
                        payload: Some(JiopadPayloadOps::from(request).to_error_response(GrpcServerError::MethodNotImplemented.into())),
                    }),
                    None => Err(GrpcServerError::InvalidRequestPayload),
                }
            })
        }));
        Self { server_ctx, methods: Default::default(), method_not_implemented }
    }

    pub fn method(&mut self, op: JiopadPayloadOps, method: JiopadMethod) {
        let method: DynJiopadMethod = Arc::new(method);
        if self.methods.insert(op, method).is_some() {
            panic!("RPC method {op:?} is declared multiple times")
        }
    }

    pub fn replace_method(&mut self, op: JiopadPayloadOps, method: JiopadMethod) {
        let method: DynJiopadMethod = Arc::new(method);
        let _ = self.methods.insert(op, method);
    }

    pub fn set_method_properties(
        &mut self,
        op: JiopadPayloadOps,
        tasks: usize,
        queue_size: usize,
        routing_policy: JiopadRoutingPolicy,
    ) {
        self.methods.entry(op).and_modify(|x| {
            let method: Method<ServerContext, Connection, JiopadRequest, JiopadResponse> =
                Method::with_properties(x.method_fn(), tasks, queue_size, routing_policy);
            let method: Arc<dyn MethodTrait<ServerContext, Connection, JiopadRequest, JiopadResponse>> = Arc::new(method);
            *x = method;
        });
    }

    pub async fn call(
        &self,
        op: &JiopadPayloadOps,
        connection: Connection,
        request: JiopadRequest,
    ) -> GrpcServerResult<JiopadResponse> {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).call(self.server_ctx.clone(), connection, request).await
    }

    pub fn get_method(&self, op: &JiopadPayloadOps) -> DynJiopadMethod {
        self.methods.get(op).unwrap_or(&self.method_not_implemented).clone()
    }
}

impl Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interface").finish()
    }
}
