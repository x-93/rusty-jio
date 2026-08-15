use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone, Default, Debug)]
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

    pub fn trigger(&self) {
        if !self.triggered.swap(true, Ordering::SeqCst) {
            self.notify.notify_waiters();
        }
    }

    pub fn is_triggered(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    pub async fn listener(&self) {
        if self.is_triggered() {
            return;
        }
        self.notify.notified().await;
    }
}

#[derive(Clone, Default, Debug)]
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

    pub fn request(&self) -> &SingleTrigger {
        &self.request
    }

    pub fn response(&self) -> &SingleTrigger {
        &self.response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_trigger() {
        let trigger = SingleTrigger::new();
        assert!(!trigger.is_triggered());

        let t_clone = trigger.clone();
        tokio::spawn(async move {
            t_clone.trigger();
        });

        trigger.listener().await;
        assert!(trigger.is_triggered());
    }
}
