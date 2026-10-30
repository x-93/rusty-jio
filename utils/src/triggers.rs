use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

/// A thread-safe shutdown signal ("alarm bell") shared between background tasks.
/// Safely stores whether it has been triggered, so late listeners do not deadlock.
#[derive(Clone)]
pub struct SingleTrigger {
    notify: Arc<Notify>,
    triggered: Arc<AtomicBool>,
}

impl Default for SingleTrigger {
    fn default() -> Self {
        Self::new()
    }
}

impl SingleTrigger {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            triggered: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Ring the bell — wakes up every task waiting for shutdown.
    pub fn trigger(&self) {
        self.triggered.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    /// Check if the trigger has already been activated.
    pub fn is_triggered(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    /// Reset the trigger back to un-triggered state.
    pub fn reset(&self) {
        self.triggered.store(false, Ordering::SeqCst);
    }

    /// Wait asynchronously until someone rings the bell.
    /// Safely registers for notification before checking the flag to avoid race conditions.
    pub async fn wait(&self) {
        // Register interest first to avoid race condition where trigger() is called between
        // checking is_triggered() and awaiting notification.
        let notified = self.notify.notified();
        if self.is_triggered() {
            return;
        }
        notified.await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trigger_before_wait() {
        let trigger = SingleTrigger::new();
        assert!(!trigger.is_triggered());
        trigger.trigger();
        assert!(trigger.is_triggered());
        // Should return immediately without hanging
        trigger.wait().await;
    }

    #[tokio::test]
    async fn test_trigger_wakes_waiter() {
        let trigger = SingleTrigger::new();
        let trigger_clone = trigger.clone();

        let handle = tokio::spawn(async move {
            trigger_clone.wait().await;
            true
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        trigger.trigger();

        let res = handle.await.unwrap();
        assert!(res);
    }

    #[tokio::test]
    async fn test_trigger_reset() {
        let trigger = SingleTrigger::new();
        trigger.trigger();
        assert!(trigger.is_triggered());
        trigger.reset();
        assert!(!trigger.is_triggered());
    }
}
