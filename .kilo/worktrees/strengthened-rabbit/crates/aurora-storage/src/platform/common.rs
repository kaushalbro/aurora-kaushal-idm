use aurora_core::error::{Result, StorageError};
use std::fs::File;
use std::io::Write;

/// Portable file preallocation by setting length and writing zero blocks if necessary.
pub fn common_preallocate(file: &File, size: u64) -> Result<()> {
    file.set_len(size).map_err(|e| StorageError::PreallocationFailed {
        size,
        reason: format!("set_len failed: {}", e),
    })?;
    Ok(())
}

/// Fallback positioned write using seek and write.
pub fn common_write_at(file: &mut File, offset: u64, data: &[u8]) -> Result<usize> {
    use std::io::Seek;
    use std::io::SeekFrom;

    file.seek(SeekFrom::Start(offset))
        .map_err(StorageError::Io)?;
    let written = file.write(data).map_err(StorageError::Io)?;
    Ok(written)
}
