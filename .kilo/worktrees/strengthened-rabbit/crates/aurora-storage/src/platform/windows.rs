use aurora_core::error::{Result, StorageError};
use std::fs::File;

#[cfg(target_os = "windows")]
use std::os::windows::fs::FileExt;

/// Windows file preallocation.
pub fn windows_preallocate(file: &File, size: u64) -> Result<()> {
    file.set_len(size).map_err(|e| StorageError::PreallocationFailed {
        size,
        reason: format!("windows set_len failed: {}", e),
    })?;
    Ok(())
}

/// Windows positioned write using `FileExt::seek_write`.
#[cfg(target_os = "windows")]
pub fn windows_write_all_at(file: &File, mut offset: u64, mut data: &[u8]) -> Result<()> {
    while !data.is_empty() {
        let written = file.seek_write(data, offset).map_err(StorageError::Io)?;
        if written == 0 {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "failed to write any bytes on Windows",
            )).into());
        }
        offset += written as u64;
        data = &data[written..];
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn windows_write_all_at(_file: &File, _offset: u64, _data: &[u8]) -> Result<()> {
    Err(StorageError::Io(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "windows_write_all_at called on non-windows target",
    )).into())
}
