pub mod counters;
pub mod monitor;
pub mod service;

pub use counters::*;
pub use monitor::*;
pub use service::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_counters() {
        let monitor = PerformanceMonitor::new();
        monitor.counters.record_block(150, 25);
        monitor.counters.record_header();

        let (blocks, txs, headers) = monitor.snapshot();
        assert_eq!(blocks, 1);
        assert_eq!(txs, 150);
        assert_eq!(headers, 1);
    }
}
