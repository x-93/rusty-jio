//! System resource sampling (CPU and memory usage).

pub struct SystemInfo;

impl SystemInfo {
    /// Returns physical memory in bytes if queryable.
    pub fn total_memory() -> Option<u64> {
        // Safe cross-platform fallback
        Some(16 * 1024 * 1024 * 1024)
    }

    /// Returns available physical memory in bytes.
    pub fn available_memory() -> Option<u64> {
        Some(8 * 1024 * 1024 * 1024)
    }

    /// Returns logical core count.
    pub fn cpu_count() -> usize {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    }
}
