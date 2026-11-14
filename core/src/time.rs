use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Returns the current Unix timestamp in milliseconds.
#[inline(always)]
pub fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_millis() as u64
}

/// Returns the current Unix timestamp in seconds.
#[inline(always)]
pub fn unix_now_sec() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}

/// Monotonic stopwatch for benchmarking consensus validation latency.
#[derive(Clone, Copy, Debug)]
pub struct Stopwatch(Instant);

impl Stopwatch {
    #[inline(always)]
    pub fn start_new() -> Self {
        Self(Instant::now())
    }

    #[inline(always)]
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }

    #[inline(always)]
    pub fn elapsed_millis(&self) -> u64 {
        self.0.elapsed().as_millis() as u64
    }

    #[inline(always)]
    pub fn elapsed_micros(&self) -> u128 {
        self.0.elapsed().as_micros()
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::start_new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_timestamps() {
        assert!(unix_now_ms() > 1_700_000_000_000);
        assert!(unix_now_sec() > 1_700_000_000);
    }

    #[test]
    fn test_stopwatch() {
        let sw = Stopwatch::start_new();
        std::thread::sleep(Duration::from_millis(5));
        assert!(sw.elapsed_millis() >= 4);
    }
}
