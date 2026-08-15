use crate::events::Notification;
use crate::scope::Scope;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub type ListenerId = u64;

pub struct Listener {
    pub id: ListenerId,
    pub sender: Sender<Notification>,
    pub subscriptions: Arc<RwLock<HashSet<Scope>>>,
}

impl Listener {
    pub fn new(id: ListenerId, sender: Sender<Notification>) -> Self {
        Self {
            id,
            sender,
            subscriptions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn subscribe(&self, scope: Scope) {
        self.subscriptions.write().insert(scope);
    }

    pub fn unsubscribe(&self, scope: &Scope) {
        self.subscriptions.write().remove(scope);
    }

    pub fn is_subscribed(&self, notification: &Notification) -> bool {
        let event_type = notification.event_type();
        let subs = self.subscriptions.read();
        subs.iter().any(|s| s.event_type() == event_type)
    }
}
