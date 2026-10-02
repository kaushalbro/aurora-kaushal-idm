use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuroraError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Download error: {0}")]
    Download(#[from] DownloadError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("HTTP error: {0}")]
    Http(#[from] HttpError),

    #[error("Recovery error: {0}")]
    Recovery(#[from] RecoveryError),

    #[error("Integrity error: {0}")]
    Integrity(#[from] IntegrityError),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Server does not support HTTP Range requests")]
    RangeUnsupported,

    #[error("Requested range {requested} does not match server response {actual}")]
    RangeMismatch { requested: String, actual: String },

    #[error("Remote resource changed during download (ETag or Last-Modified mismatch)")]
    ResourceChanged,

    #[error("Exceeded maximum retries ({attempts}): {reason}")]
    TooManyRetries { attempts: u32, reason: String },

    #[error("Download was cancelled")]
    Cancelled,

    #[error("Download is paused")]
    Paused,

    #[error("Invalid download state transition: {0}")]
    InvalidState(String),

    #[error("Worker {0} stopped unexpectedly")]
    WorkerFailed(u32),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to preallocate {size} bytes on disk: {reason}")]
    PreallocationFailed { size: u64, reason: String },

    #[error("Disk full or insufficient space (required {required} bytes, available {available} bytes)")]
    DiskFull { required: u64, available: u64 },

    #[error("Invalid write offset {offset} for file size {size}")]
    InvalidOffset { offset: u64, size: u64 },

    #[error("Storage writer channel is closed")]
    ChannelClosed,
}

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Network connection error: {0}")]
    Network(String),

    #[error("Connection or read timed out")]
    Timeout,

    #[error("HTTP {status}: {message}")]
    HttpStatus {
        status: u16,
        message: String,
        retry_after: Option<std::time::Duration>,
    },

    #[error("Invalid HTTP response header: {0}")]
    InvalidHeader(String),

    #[error("Too many redirects: {0}")]
    RedirectLoop(String),
}

#[derive(Error, Debug)]
pub enum RecoveryError {
    #[error("Corrupt recovery journal: {0}")]
    CorruptJournal(String),

    #[error("Resume validation failed: {0}")]
    ValidationFailed(String),

    #[error("Destination file not found or corrupted during resume: {0}")]
    FileNotFound(String),
}

#[derive(Error, Debug)]
pub enum IntegrityError {
    #[error("Checksum mismatch for {algorithm} (expected {expected}, actual {actual})")]
    ChecksumMismatch {
        algorithm: String,
        expected: String,
        actual: String,
    },

    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

pub type Result<T> = std::result::Result<T, AuroraError>;
