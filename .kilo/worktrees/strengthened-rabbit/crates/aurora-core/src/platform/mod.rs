pub mod common;
pub mod linux;
pub mod windows;

use std::path::PathBuf;

/// Returns the standard system Downloads directory.
pub fn default_download_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        linux::linux_download_dir()
    }
    #[cfg(target_os = "windows")]
    {
        windows::windows_download_dir()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        common::fallback_download_dir()
    }
}

/// Returns the application configuration directory.
pub fn default_config_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        linux::linux_config_dir()
    }
    #[cfg(target_os = "windows")]
    {
        windows::windows_config_dir()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        common::fallback_config_dir()
    }
}

/// Returns the application data directory.
pub fn default_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        linux::linux_data_dir()
    }
    #[cfg(target_os = "windows")]
    {
        windows::windows_data_dir()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        common::fallback_data_dir()
    }
}
