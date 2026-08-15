use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub cpu_usage_pct: f64,
    pub resident_memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub thread_count: usize,
    pub open_file_descriptors: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_memory_bytes: u64,
    pub free_memory_bytes: u64,
    pub cpu_cores: usize,
}
