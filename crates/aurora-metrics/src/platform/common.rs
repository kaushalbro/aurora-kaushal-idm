/// Fallback process resource metrics.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessMetrics {
    pub resident_memory_bytes: u64,
    pub cpu_usage_pct: f32,
}

pub fn common_process_metrics() -> ProcessMetrics {
    ProcessMetrics {
        resident_memory_bytes: 0,
        cpu_usage_pct: 0.0,
    }
}
