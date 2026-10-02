//! Simulation and virtual clock utilities.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Default)]
pub struct VirtualClock {
    current_time_ms: AtomicU64,
}

impl VirtualClock {
    pub fn new(initial_time_ms: u64) -> Self {
        Self {
            current_time_ms: AtomicU64::new(initial_time_ms),
        }
    }

    pub fn now_ms(&self) -> u64 {
        self.current_time_ms.load(Ordering::SeqCst)
    }

    pub fn advance(&self, duration: Duration) {
        self.current_time_ms.fetch_add(duration.as_millis() as u64, Ordering::SeqCst);
    }
}
