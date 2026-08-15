use crate::monitor::PerformanceMonitor;
use std::sync::Arc;

pub struct PerformanceMonitorService {
    monitor: Arc<PerformanceMonitor>,
}

impl PerformanceMonitorService {
    pub fn new(monitor: Arc<PerformanceMonitor>) -> Self {
        Self { monitor }
    }

    pub fn monitor(&self) -> Arc<PerformanceMonitor> {
        self.monitor.clone()
    }
}
