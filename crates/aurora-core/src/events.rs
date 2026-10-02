use crate::types::{ByteRange, DownloadId, DownloadStatus, SegmentId, WorkerId};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;

/// Real-time lifecycle events emitted by the download engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadEvent {
    /// Download task was added to queue.
    Queued { download_id: DownloadId },
    /// Probing started for remote capability discovery.
    ProbingStarted { download_id: DownloadId },
    /// Remote server capabilities and metadata loaded.
    MetadataLoaded {
        download_id: DownloadId,
        content_length: Option<u64>,
        accepts_ranges: bool,
        etag: Option<String>,
        suggested_filename: String,
        rtt_ms: Option<u64>,
        server_name: Option<String>,
        health_rating: Option<String>,
    },
    /// Download worker thread/stream initiated.
    WorkerStarted {
        download_id: DownloadId,
        worker_id: WorkerId,
        assigned_range: ByteRange,
    },
    /// Download worker terminated or returned to pool.
    WorkerStopped {
        download_id: DownloadId,
        worker_id: WorkerId,
        bytes_transferred: u64,
    },
    /// Segment began active downloading.
    SegmentStarted {
        download_id: DownloadId,
        segment_id: SegmentId,
        range: ByteRange,
        worker_id: WorkerId,
    },
    /// Progress update for a segment.
    SegmentProgress {
        download_id: DownloadId,
        segment_id: SegmentId,
        bytes_downloaded: u64,
        total_segment_bytes: u64,
    },
    /// Segment completed transfer and written to storage.
    SegmentCompleted {
        download_id: DownloadId,
        segment_id: SegmentId,
        bytes_written: u64,
    },
    /// Segment split by dynamic scheduler.
    SegmentSplit {
        download_id: DownloadId,
        original_segment_id: SegmentId,
        new_segment_id: SegmentId,
        left_range: ByteRange,
        right_range: ByteRange,
        reason: String,
    },
    /// Global or per-download speed & throughput update.
    ThroughputUpdated {
        download_id: DownloadId,
        instantaneous_rate: f64,
        smoothed_rate: f64,
        eta: Option<Duration>,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
    },
    /// Storage flush / backpressure update.
    StoragePressure {
        download_id: DownloadId,
        pending_queue_bytes: usize,
        disk_write_rate: f64,
        is_backpressure_active: bool,
    },
    /// State transitions.
    StatusChanged {
        download_id: DownloadId,
        new_status: DownloadStatus,
    },
    /// Worker or segment retrying after transient error.
    Retrying {
        download_id: DownloadId,
        segment_id: SegmentId,
        attempt: u32,
        delay: Duration,
        reason: String,
    },
    /// Checkpoint saved to disk.
    CheckpointSaved {
        download_id: DownloadId,
        saved_bytes: u64,
    },
    /// Download fully completed and verified.
    Completed {
        download_id: DownloadId,
        total_bytes: u64,
        elapsed: Duration,
        average_speed: f64,
        checksum: Option<String>,
    },
    /// Download failed permanently.
    Failed {
        download_id: DownloadId,
        error: String,
    },
}

pub type EventSender = broadcast::Sender<DownloadEvent>;
pub type EventReceiver = broadcast::Receiver<DownloadEvent>;

pub fn create_event_bus(capacity: usize) -> (EventSender, EventReceiver) {
    let (tx, rx) = broadcast::channel(capacity);
    (tx, rx)
}
