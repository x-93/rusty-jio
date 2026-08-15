use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time after UNIX epoch")
        .as_millis() as u64
}

pub fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time after UNIX epoch")
        .as_secs()
}

#[derive(Debug, Clone)]
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    pub fn start_new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_millis(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::start_new()
    }
}
