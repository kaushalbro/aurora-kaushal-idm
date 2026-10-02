pub mod common;
pub mod linux;
pub mod windows;

pub use common::ProcessMetrics;

pub fn current_process_metrics() -> ProcessMetrics {
    #[cfg(target_os = "linux")]
    {
        linux::linux_process_metrics()
    }
    #[cfg(target_os = "windows")]
    {
        windows::windows_process_metrics()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        common::common_process_metrics()
    }
}
