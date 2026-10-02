use aurora_core::error::{IntegrityError, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

impl HashAlgorithm {
    pub fn from_str_name(name: &str) -> std::result::Result<Self, IntegrityError> {
        match name.to_lowercase().trim() {
            "sha256" | "sha-256" => Ok(Self::Sha256),
            "blake3" => Ok(Self::Blake3),
            other => Err(IntegrityError::UnsupportedAlgorithm(other.to_string())),
        }
    }
}

/// Verifies a file on disk against an expected hexadecimal checksum string.
pub fn verify_file_checksum(
    path: &Path,
    expected_hex: &str,
    algorithm: HashAlgorithm,
) -> Result<bool> {
    let file = File::open(path).map_err(aurora_core::error::StorageError::Io)?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file); // 1 MB buffer
    let mut buffer = [0u8; 64 * 1024];

    let actual_hex = match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            loop {
                let n = reader.read(&mut buffer).map_err(aurora_core::error::StorageError::Io)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            format!("{:x}", hasher.finalize())
        }
        HashAlgorithm::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let n = reader.read(&mut buffer).map_err(aurora_core::error::StorageError::Io)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            hasher.finalize().to_hex().to_string()
        }
    };

    let expected_clean = expected_hex.trim().to_lowercase();
    if actual_hex.eq_ignore_ascii_case(&expected_clean) {
        Ok(true)
    } else {
        Err(IntegrityError::ChecksumMismatch {
            algorithm: format!("{:?}", algorithm),
            expected: expected_clean,
            actual: actual_hex,
        }
        .into())
    }
}

/// Computes the SHA-256 checksum for an entire file.
pub fn compute_sha256(path: &Path) -> Result<String> {
    let file = File::open(path).map_err(aurora_core::error::StorageError::Io)?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = reader.read(&mut buffer).map_err(aurora_core::error::StorageError::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
