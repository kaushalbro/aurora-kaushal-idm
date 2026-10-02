pub mod common;
pub mod linux;
pub mod windows;

use aurora_core::error::Result;
use std::fs::File;

/// Preallocates `size` bytes on disk for `file`.
pub fn preallocate_file(file: &File, size: u64) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::linux_preallocate(file, size)
    }
    #[cfg(target_os = "windows")]
    {
        windows::windows_preallocate(file, size)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        common::common_preallocate(file, size)
    }
}

/// Positioned write at specific byte offset.
pub fn write_all_at(file: &File, offset: u64, data: &[u8]) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::linux_write_all_at(file, offset, data)
    }
    #[cfg(target_os = "windows")]
    {
        windows::windows_write_all_at(file, offset, data)
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        // For fallback, use mutex or handle-based sequential write
        let mut file_clone = file.try_clone().map_err(aurora_core::error::StorageError::Io)?;
        common::common_write_at(&mut file_clone, offset, data)?;
        Ok(())
    }
}
