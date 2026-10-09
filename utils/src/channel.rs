//! Bounded channels with backpressure and throughput metrics.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, error::TrySendError};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ChannelError<T> {
    #[error("channel is closed")]
    Closed(T),
    #[error("channel is full")]
    Full(T),
    #[error("operation timed out")]
    Timeout(T),
}

/// Statistics and counters for a bounded channel.
#[derive(Default, Debug)]
pub struct ChannelMetrics {
    sent: AtomicUsize,
    received: AtomicUsize,
    dropped: AtomicUsize,
}

impl ChannelMetrics {
    pub fn total_sent(&self) -> usize {
        self.sent.load(Ordering::Relaxed)
    }

    pub fn total_received(&self) -> usize {
        self.received.load(Ordering::Relaxed)
    }

    pub fn total_dropped(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// Bounded channel sender with queue metrics.
#[derive(Clone)]
pub struct ChannelSender<T> {
    inner: mpsc::Sender<T>,
    metrics: Arc<ChannelMetrics>,
}

impl<T> ChannelSender<T> {
    pub async fn send(&self, msg: T) -> Result<(), ChannelError<T>> {
        self.inner.send(msg).await.map_err(|e| {
            self.metrics.dropped.fetch_add(1, Ordering::Relaxed);
            ChannelError::Closed(e.0)
        })?;
        self.metrics.sent.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn try_send(&self, msg: T) -> Result<(), ChannelError<T>> {
        match self.inner.try_send(msg) {
            Ok(()) => {
                self.metrics.sent.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(TrySendError::Full(val)) => {
                self.metrics.dropped.fetch_add(1, Ordering::Relaxed);
                Err(ChannelError::Full(val))
            }
            Err(TrySendError::Closed(val)) => {
                self.metrics.dropped.fetch_add(1, Ordering::Relaxed);
                Err(ChannelError::Closed(val))
            }
        }
    }

    pub async fn send_timeout(&self, msg: T, timeout: Duration) -> Result<(), ChannelError<T>> {
        match tokio::time::timeout(timeout, self.inner.send(msg)).await {
            Ok(Ok(())) => {
                self.metrics.sent.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Ok(Err(e)) => {
                self.metrics.dropped.fetch_add(1, Ordering::Relaxed);
                Err(ChannelError::Closed(e.0))
            }
            Err(_) => {
                panic!("Timeout while sending to channel");
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

/// Bounded channel receiver with queue metrics.
pub struct ChannelReceiver<T> {
    inner: mpsc::Receiver<T>,
    metrics: Arc<ChannelMetrics>,
}

impl<T> ChannelReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        let msg = self.inner.recv().await;
        if msg.is_some() {
            self.metrics.received.fetch_add(1, Ordering::Relaxed);
        }
        msg
    }

    pub fn try_recv(&mut self) -> Result<T, tokio::sync::mpsc::error::TryRecvError> {
        let res = self.inner.try_recv();
        if res.is_ok() {
            self.metrics.received.fetch_add(1, Ordering::Relaxed);
        }
        res
    }

    pub fn metrics(&self) -> &ChannelMetrics {
        &self.metrics
    }
}

/// Create a bounded channel with given capacity and metrics tracking.
pub fn bounded<T>(capacity: usize) -> (ChannelSender<T>, ChannelReceiver<T>, Arc<ChannelMetrics>) {
    let (tx, rx) = mpsc::channel(capacity);
    let metrics = Arc::new(ChannelMetrics::default());
    let sender = ChannelSender {
        inner: tx,
        metrics: Arc::clone(&metrics),
    };
    let receiver = ChannelReceiver {
        inner: rx,
        metrics: Arc::clone(&metrics),
    };
    (sender, receiver, metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bounded_channel() {
        let (tx, mut rx, metrics) = bounded::<u32>(2);
        assert_eq!(tx.capacity(), 2);

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        assert!(matches!(tx.try_send(3), Err(ChannelError::Full(3))));

        assert_eq!(metrics.total_sent(), 2);
        assert_eq!(metrics.total_dropped(), 1);

        assert_eq!(rx.recv().await, Some(1));
        assert_eq!(rx.recv().await, Some(2));
        assert_eq!(metrics.total_received(), 2);
    }
}