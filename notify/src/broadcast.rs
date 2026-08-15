use crate::events::Notification;
use crate::listener::Listener;
use std::sync::Arc;

pub struct BroadcastManager;

impl BroadcastManager {
    pub async fn broadcast(listeners: &[Arc<Listener>], notification: Notification) {
        for listener in listeners {
            if listener.is_subscribed(&notification) {
                let _ = listener.sender.send(notification.clone()).await;
            }
        }
    }
}
