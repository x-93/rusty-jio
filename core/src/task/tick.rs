//! Cooperative green-task ticks and yield primitives for long-running tasks.

use std::time::Duration;
use tokio::time::interval;

/// Yield execution cooperatively to the Tokio executor every `yield_interval` ticks.
#[inline(always)]
pub async fn cooperative_tick(counter: &mut usize, yield_interval: usize) {
    *counter += 1;
    if *counter >= yield_interval {
        *counter = 0;
        tokio::task::yield_now().await;
    }
}

/// Periodic cooperative tick generator.
pub struct TickService {
    interval_duration: Duration,
}

impl TickService {
    pub fn new(interval_duration: Duration) -> Self {
        Self { interval_duration }
    }

    /// Run the ticking loop until cancellation token fires, calling `on_tick` at each tick.
    pub async fn run<F>(&self, token: tokio_util::sync::CancellationToken, mut on_tick: F)
    where
        F: FnMut() + Send + 'static,
    {
        let mut interval = interval(self.interval_duration);
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break;
                }
                _ = interval.tick() => {
                    on_tick();
                    tokio::task::yield_now().await;
                }
            }
        }
    }
}
