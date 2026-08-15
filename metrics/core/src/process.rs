use crate::data::ProcessMetrics;

pub trait ProcessMetricsCollector: Send + Sync {
    fn collect(&self) -> ProcessMetrics;
}

#[derive(Default)]
pub struct DefaultProcessCollector;

impl ProcessMetricsCollector for DefaultProcessCollector {
    fn collect(&self) -> ProcessMetrics {
        ProcessMetrics {
            cpu_usage_pct: 0.0,
            resident_memory_bytes: 50 * 1024 * 1024,
            virtual_memory_bytes: 100 * 1024 * 1024,
            thread_count: 8,
            open_file_descriptors: 32,
        }
    }
}
