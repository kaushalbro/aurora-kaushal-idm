use crate::state::ResumeState;
use aurora_core::error::{RecoveryError, Result, StorageError};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Computes the standard journal metadata path for a destination file.
pub fn journal_path_for_file(destination_path: &Path) -> PathBuf {
    let mut s = destination_path.as_os_str().to_os_string();
    s.push(".aurora.state");
    PathBuf::from(s)
}

/// Atomically persists download state to disk using a write-flush-rename sequence.
pub fn save_journal(state: &ResumeState) -> Result<PathBuf> {
    let journal_path = journal_path_for_file(&state.destination_path);
    let tmp_path = {
        let mut s = journal_path.as_os_str().to_os_string();
        s.push(".tmp");
        PathBuf::from(s)
    };

    if let Some(parent) = journal_path.parent() {
        fs::create_dir_all(parent).map_err(StorageError::Io)?;
    }

    let json_bytes = serde_json::to_vec_pretty(state)
        .map_err(|e| RecoveryError::CorruptJournal(format!("Serialization error: {}", e)))?;

    // Step 1: Write to temporary file
    {
        let mut file = File::create(&tmp_path).map_err(StorageError::Io)?;
        file.write_all(&json_bytes).map_err(StorageError::Io)?;
        file.sync_all().map_err(StorageError::Io)?;
    }

    // Step 2: Atomic rename over the target journal file
    fs::rename(&tmp_path, &journal_path).map_err(StorageError::Io)?;

    debug!(
        "Persisted atomic checkpoint to {:?} (downloaded: {} bytes)",
        journal_path,
        state.total_downloaded_bytes()
    );

    Ok(journal_path)
}

/// Loads and deserializes resume state from disk, with fallback recovery.
pub fn load_journal(journal_path: &Path) -> Result<ResumeState> {
    if !journal_path.exists() {
        // Check if there's a leftover tmp file
        let tmp_path = {
            let mut s = journal_path.as_os_str().to_os_string();
            s.push(".tmp");
            PathBuf::from(s)
        };
        if tmp_path.exists() {
            warn!("Journal file missing, recovering from tmp file: {:?}", tmp_path);
            let _ = fs::rename(&tmp_path, journal_path);
        }
    }

    if !journal_path.exists() {
        return Err(RecoveryError::FileNotFound(format!(
            "Journal file does not exist: {:?}",
            journal_path
        ))
        .into());
    }

    let bytes = fs::read(journal_path).map_err(StorageError::Io)?;
    let state: ResumeState = serde_json::from_slice(&bytes)
        .map_err(|e| RecoveryError::CorruptJournal(format!("Deserialization error: {}", e)))?;

    info!(
        "Successfully loaded recovery journal for {:?} ({} segments)",
        state.destination_path,
        state.segments.len()
    );

    Ok(state)
}

/// Safely removes the journal file after download finalization.
pub fn remove_journal(destination_path: &Path) -> Result<()> {
    let journal_path = journal_path_for_file(destination_path);
    if journal_path.exists() {
        let _ = fs::remove_file(&journal_path);
    }
    let tmp_path = {
        let mut s = journal_path.as_os_str().to_os_string();
        s.push(".tmp");
        PathBuf::from(s)
    };
    if tmp_path.exists() {
        let _ = fs::remove_file(&tmp_path);
    }
    Ok(())
}
