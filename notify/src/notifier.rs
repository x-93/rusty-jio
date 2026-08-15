use crate::broadcast::BroadcastManager;
use crate::events::Notification;
use crate::listener::{Listener, ListenerId};
use crate::scope::Scope;
use crate::subscriber::Subscriber;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver};

#[derive(Clone)]
pub struct Notifier {
    listeners: Arc<RwLock<HashMap<ListenerId, Arc<Listener>>>>,
    next_listener_id: Arc<AtomicU64>,
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            next_listener_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn register_listener(&self) -> (ListenerId, Receiver<Notification>) {
        let id = self.next_listener_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = channel(256);
        let listener = Arc::new(Listener::new(id, tx));
        self.listeners.write().insert(id, listener);
        (id, rx)
    }

    pub fn unregister_listener(&self, id: ListenerId) {
        self.listeners.write().remove(&id);
    }

    pub async fn notify(&self, notification: Notification) {
        let listeners: Vec<_> = self.listeners.read().values().cloned().collect();
        BroadcastManager::broadcast(&listeners, notification).await;
    }
}

#[async_trait]
impl Subscriber for Notifier {
    async fn subscribe(&self, listener_id: ListenerId, scope: Scope) {
        if let Some(listener) = self.listeners.read().get(&listener_id) {
            listener.subscribe(scope);
        }
    }

    async fn unsubscribe(&self, listener_id: ListenerId, scope: Scope) {
        if let Some(listener) = self.listeners.read().get(&listener_id) {
            listener.unsubscribe(&scope);
        }
    }
}
