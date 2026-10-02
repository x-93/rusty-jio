//! Asynchronous trigger primitives for one-shot signaling and shutdown coordination.

use tokio::sync::Notify;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A one-shot asynchronous trigger.
#[derive(Clone, Default)]
pub struct SingleTrigger {
    triggered: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl SingleTrigger {
    pub fn new() -> Self {
        Self {
            triggered: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Triggers the event and notifies all current waiters.
    pub fn trigger(&self) {
        if !self.triggered.swap(true, Ordering::SeqCst) {
            self.notify.notify_waiters();
        }
    }

    /// Returns true if already triggered.
    pub fn is_triggered(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    /// Awaits until triggered.
    pub async fn listener(&self) {
        if self.is_triggered() {
            return;
        }
        self.notify.notified().await;
    }
}

/// A duplex trigger facilitating bidirectional signaling between requester and responder.
#[derive(Clone, Default)]
pub struct DuplexTrigger {
    request: SingleTrigger,
    response: SingleTrigger,
}

impl DuplexTrigger {
    pub fn new() -> Self {
        Self {
            request: SingleTrigger::new(),
            response: SingleTrigger::new(),
        }
    }

    pub fn request(&self) {
        self.request.trigger();
    }

    pub async fn wait_for_request(&self) {
        self.request.listener().await;
    }

    pub fn respond(&self) {
        self.response.trigger();
    }

    pub async fn wait_for_response(&self) {
        self.response.listener().await;
    }
}
