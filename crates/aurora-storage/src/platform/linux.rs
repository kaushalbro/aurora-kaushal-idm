use aurora_core::error::{Result, StorageError};
use std::fs::File;

#[cfg(target_family = "unix")]
use std::os::unix::fs::FileExt;

/// Linux file preallocation using set_len and POSIX hints.
pub fn linux_preallocate(file: &File, size: u64) -> Result<()> {
    file.set_len(size).map_err(|e| StorageError::PreallocationFailed {
        size,
        reason: format!("linux set_len failed: {}", e),
    })?;
    Ok(())
}

/// Linux positioned write using `FileExt::write_all_at` (pwrite under the hood, thread-safe, no seek needed).
#[cfg(target_family = "unix")]
pub fn linux_write_all_at(file: &File, offset: u64, data: &[u8]) -> Result<()> {
    file.write_all_at(data, offset).map_err(StorageError::Io)?;
    Ok(())
}

#[cfg(not(target_family = "unix"))]
pub fn linux_write_all_at(_file: &File, _offset: u64, _data: &[u8]) -> Result<()> {
    Err(StorageError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "linux_write_all_at called on non-unix target",
    )).into())
}
