use super::common::ProcessMetrics;

pub fn windows_process_metrics() -> ProcessMetrics {
    // Basic Windows metric query or fallback
    crate::platform::common::common_process_metrics()
}
