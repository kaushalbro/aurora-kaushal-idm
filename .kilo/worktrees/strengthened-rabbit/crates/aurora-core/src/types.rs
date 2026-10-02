use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, Instant};
use url::Url;
use uuid::Uuid;

/// Unique identifier for a download task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DownloadId(pub Uuid);

impl DownloadId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for DownloadId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DownloadId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a segment within a download.
pub type SegmentId = u32;

/// Unique identifier for an active network worker.
pub type WorkerId = u32;

/// Represents an inclusive byte range [start, end].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

impl ByteRange {
    #[inline]
    pub fn new(start: u64, end: u64) -> Self {
        debug_assert!(start <= end, "start must be <= end");
        Self { start, end }
    }

    #[inline]
    pub fn len(&self) -> u64 {
        if self.end >= self.start {
            self.end - self.start + 1
        } else {
            0
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn contains(&self, offset: u64) -> bool {
        offset >= self.start && offset <= self.end
    }

    /// Splits this range at an absolute split offset `split_at`.
    /// Left range becomes [start, split_at - 1], right range becomes [split_at, end].
    /// Returns None if split_at <= start or split_at > end.
    pub fn split_at(&self, split_at: u64) -> Option<(ByteRange, ByteRange)> {
        if split_at <= self.start || split_at > self.end {
            return None;
        }
        Some((
            ByteRange::new(self.start, split_at - 1),
            ByteRange::new(split_at, self.end),
        ))
    }

    /// Splits the range according to a proportional ratio for left side (0.0 < ratio < 1.0).
    pub fn split_ratio(&self, left_ratio: f64) -> Option<(ByteRange, ByteRange)> {
        if left_ratio <= 0.0 || left_ratio >= 1.0 || self.len() < 2 {
            return None;
        }
        let total_bytes = self.len() as f64;
        let left_bytes = (total_bytes * left_ratio).round() as u64;
        let left_bytes = left_bytes.clamp(1, self.len() - 1);
        let split_offset = self.start + left_bytes;
        self.split_at(split_offset)
    }

    /// Formats the range for standard HTTP Range header (e.g., "bytes=0-1024").
    pub fn to_http_header(&self) -> String {
        format!("bytes={}-{}", self.start, self.end)
    }
}

impl fmt::Display for ByteRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}-{}] ({} bytes)", self.start, self.end, self.len())
    }
}

/// Lifecycle state of an individual download segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentState {
    Pending,
    Downloading,
    Completed,
    Failed,
    Paused,
}

/// Represents a discrete segment of a file download.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub id: SegmentId,
    pub range: ByteRange,
    pub downloaded: u64,
    pub state: SegmentState,
    pub worker_id: Option<WorkerId>,
    pub retry_count: u32,
    pub error_reason: Option<String>,
}

impl Segment {
    pub fn new(id: SegmentId, range: ByteRange) -> Self {
        Self {
            id,
            range,
            downloaded: 0,
            state: SegmentState::Pending,
            worker_id: None,
            retry_count: 0,
            error_reason: None,
        }
    }

    #[inline]
    pub fn remaining_bytes(&self) -> u64 {
        self.range.len().saturating_sub(self.downloaded)
    }

    #[inline]
    pub fn current_write_offset(&self) -> u64 {
        self.range.start + self.downloaded
    }

    #[inline]
    pub fn is_completed(&self) -> bool {
        self.downloaded >= self.range.len()
    }

    #[inline]
    pub fn progress_fraction(&self) -> f64 {
        let total = self.range.len();
        if total == 0 {
            1.0
        } else {
            (self.downloaded as f64 / total as f64).clamp(0.0, 1.0)
        }
    }
}

/// Operational statistics measured for an active network worker.
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub worker_id: WorkerId,
    pub bytes_downloaded: u64,
    pub instantaneous_rate: f64, // bytes/sec
    pub smoothed_rate: f64,      // EWMA bytes/sec
    pub current_rtt: Duration,
    pub min_rtt: Duration,
    pub failures: u32,
    pub retries: u32,
    pub started_at: Instant,
    pub last_progress: Instant,
}

impl WorkerStats {
    pub fn new(worker_id: WorkerId) -> Self {
        let now = Instant::now();
        Self {
            worker_id,
            bytes_downloaded: 0,
            instantaneous_rate: 0.0,
            smoothed_rate: 0.0,
            current_rtt: Duration::from_millis(50),
            min_rtt: Duration::from_millis(50),
            failures: 0,
            retries: 0,
            started_at: now,
            last_progress: now,
        }
    }

    /// Predicts remaining time for `bytes` using smoothed rate, accounting for setup overhead.
    pub fn estimate_completion_time(&self, bytes: u64, setup_overhead: Duration) -> Duration {
        if bytes == 0 {
            return Duration::ZERO;
        }
        let rate = if self.smoothed_rate > 1024.0 {
            self.smoothed_rate
        } else if self.instantaneous_rate > 1024.0 {
            self.instantaneous_rate
        } else {
            100_000.0 // Conservative default estimate: ~100 KB/s
        };
        let transfer_seconds = bytes as f64 / rate;
        Duration::from_secs_f64(transfer_seconds) + setup_overhead
    }
}

/// Remote HTTP server capabilities discovered via probing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub accepts_ranges: bool,
    pub content_length: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub mime_type: Option<String>,
    pub final_url: Url,
    pub http_version: String,
    pub is_resumable: bool,
    pub rtt_ms: Option<u64>,
    pub server_name: Option<String>,
    pub health_rating: Option<String>,
}

impl ServerCapabilities {
    pub fn fallback_single_stream(url: Url) -> Self {
        Self {
            accepts_ranges: false,
            content_length: None,
            etag: None,
            last_modified: None,
            mime_type: None,
            final_url: url,
            http_version: "HTTP/1.1".to_string(),
            is_resumable: false,
            rtt_ms: None,
            server_name: None,
            health_rating: None,
        }
    }
}

/// High-level status of a download job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Probing,
    Downloading,
    Finalizing,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

impl fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "Queued"),
            Self::Probing => write!(f, "Probing"),
            Self::Downloading => write!(f, "Downloading"),
            Self::Finalizing => write!(f, "Finalizing"),
            Self::Paused => write!(f, "Paused"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed(msg) => write!(f, "Failed: {}", msg),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Comprehensive download state snapshot.
#[derive(Debug, Clone)]
pub struct DownloadSnapshot {
    pub id: DownloadId,
    pub url: Url,
    pub filename: String,
    pub destination_path: std::path::PathBuf,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
    pub aggregate_rate: f64,
    pub eta: Option<Duration>,
    pub active_connections: usize,
    pub segments: Vec<Segment>,
    pub server_capabilities: Option<ServerCapabilities>,
}
