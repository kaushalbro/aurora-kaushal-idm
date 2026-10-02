use crate::error::Result;
use crate::types::{ByteRange, DownloadSnapshot, SegmentId, ServerCapabilities, WorkerId};
use bytes::Bytes;
use std::time::Duration;
use url::Url;

/// Next scheduling action proposed by a Scheduler implementation.
#[derive(Debug, Clone, PartialEq)]
pub enum SchedulerAction {
    /// No change needed at this time.
    DoNothing,
    /// Allocate a new segment to an idle worker.
    StartSegment {
        worker_id: WorkerId,
        range: ByteRange,
    },
    /// Split an existing active segment and assign the stolen sub-range to an idle worker.
    SplitAndAssign {
        target_segment_id: SegmentId,
        new_range_for_existing: ByteRange,
        stolen_range_for_new_worker: ByteRange,
        idle_worker_id: WorkerId,
    },
    /// Increase concurrency by spawning a new worker.
    SpawnWorker {
        target_range: ByteRange,
    },
    /// Decrease concurrency by allowing specified worker to finish and exit.
    RetireWorker {
        worker_id: WorkerId,
    },
    /// Speculatively duplicate an extreme straggler segment.
    DuplicateTail {
        source_segment_id: SegmentId,
        hedge_range: ByteRange,
        worker_id: WorkerId,
    },
}

/// Abstract transport layer interface (supporting HTTP/1.1, HTTP/2, and future HTTP/3).
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Probes the remote endpoint for capabilities (Content-Length, Accept-Ranges, ETag, etc.).
    async fn probe(&self, url: &Url) -> Result<ServerCapabilities>;
}

/// Abstract storage engine interface for direct positioned writes.
#[async_trait::async_trait]
pub trait StorageEngine: Send + Sync {
    /// Preallocates file on disk to prevent fragmentation and guarantee space.
    async fn preallocate(&mut self, size: u64) -> Result<()>;

    /// Asynchronously writes a chunk of bytes directly to the specified file offset.
    async fn write_chunk(&mut self, offset: u64, data: Bytes) -> Result<()>;

    /// Flushes uncommitted in-memory buffers to physical storage.
    async fn flush(&mut self) -> Result<()>;

    /// Verifies and finalizes destination file on disk.
    async fn finalize(&mut self) -> Result<()>;
}

/// Abstract scheduler interface.
pub trait Scheduler: Send {
    /// Evaluates current download state and returns the next scheduling action.
    fn next_action(&mut self, state: &DownloadSnapshot) -> SchedulerAction;
    
    /// Resets or reconfigures the scheduler.
    fn reset(&mut self);
}

/// Rate and bandwidth estimator.
pub trait RateEstimator: Send + Sync {
    fn update(&mut self, bytes: u64, dt: Duration);
    fn smoothed_rate(&self) -> f64;
    fn instantaneous_rate(&self) -> f64;
    fn reset(&mut self);
}
