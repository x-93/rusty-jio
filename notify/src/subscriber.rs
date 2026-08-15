use crate::listener::ListenerId;
use crate::scope::Scope;
use async_trait::async_trait;

#[async_trait]
pub trait Subscriber: Send + Sync {
    async fn subscribe(&self, listener_id: ListenerId, scope: Scope);
    async fn unsubscribe(&self, listener_id: ListenerId, scope: Scope);
}
