//! Recurring ticker service for background maintenance tasks.

use std::time::Duration;
use tokio::time::interval;

pub struct TickService;

impl TickService {
    pub async fn run_periodic<F, Fut>(period: Duration, mut task: F)
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut ticker = interval(period);
        loop {
            ticker.tick().await;
            task().await;
        }
    }
}
