use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone, Default, Debug)]
pub struct SimulationClock {
    now: Arc<AtomicU64>,
}

impl SimulationClock {
    pub fn new(initial_timestamp: u64) -> Self {
        Self {
            now: Arc::new(AtomicU64::new(initial_timestamp)),
        }
    }

    pub fn now(&self) -> u64 {
        self.now.load(Ordering::Relaxed)
    }

    pub fn advance(&self, duration_ms: u64) -> u64 {
        self.now.fetch_add(duration_ms, Ordering::SeqCst) + duration_ms
    }

    pub fn set(&self, timestamp: u64) {
        self.now.store(timestamp, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_clock() {
        let clock = SimulationClock::new(1000);
        assert_eq!(clock.now(), 1000);
        assert_eq!(clock.advance(500), 1500);
        assert_eq!(clock.now(), 1500);
    }
}
