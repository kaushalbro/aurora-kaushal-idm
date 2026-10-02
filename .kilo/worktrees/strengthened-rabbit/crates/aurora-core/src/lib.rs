pub mod config;
pub mod error;
pub mod events;
pub mod platform;
pub mod traits;
pub mod types;

pub use config::{EngineConfig, SchedulerType};
pub use error::{AuroraError, DownloadError, HttpError, IntegrityError, RecoveryError, Result, StorageError};
pub use events::{create_event_bus, DownloadEvent, EventReceiver, EventSender};
pub use traits::{RateEstimator, Scheduler, SchedulerAction, StorageEngine, Transport};
pub use types::{
    ByteRange, DownloadId, DownloadSnapshot, DownloadStatus, Segment, SegmentId, SegmentState,
    ServerCapabilities, WorkerId, WorkerStats,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_byte_range_properties() {
        let range = ByteRange::new(0, 999);
        assert_eq!(range.len(), 1000);
        assert!(!range.is_empty());
        assert!(range.contains(0));
        assert!(range.contains(500));
        assert!(range.contains(999));
        assert!(!range.contains(1000));
        assert_eq!(range.to_http_header(), "bytes=0-999");
    }

    #[test]
    fn test_byte_range_split_at_offset() {
        let range = ByteRange::new(100, 499); // 400 bytes
        let (left, right) = range.split_at(300).expect("should split at 300");

        assert_eq!(left.start, 100);
        assert_eq!(left.end, 299);
        assert_eq!(left.len(), 200);

        assert_eq!(right.start, 300);
        assert_eq!(right.end, 499);
        assert_eq!(right.len(), 200);

        // Splitting invariants:
        assert_eq!(left.start, range.start);
        assert_eq!(left.end + 1, right.start);
        assert_eq!(right.end, range.end);
        assert_eq!(left.len() + right.len(), range.len());

        // Boundary violations
        assert!(range.split_at(100).is_none());
        assert!(range.split_at(500).is_none());
    }

    #[test]
    fn test_byte_range_proportional_split() {
        let range = ByteRange::new(0, 999); // 1000 bytes
        // 25% to 75% split
        let (left, right) = range.split_ratio(0.25).expect("should split 25/75");

        assert_eq!(left.start, 0);
        assert_eq!(left.end, 249);
        assert_eq!(left.len(), 250);

        assert_eq!(right.start, 250);
        assert_eq!(right.end, 999);
        assert_eq!(right.len(), 750);

        assert_eq!(left.len() + right.len(), range.len());
        assert_eq!(left.end + 1, right.start);
    }

    #[test]
    fn test_worker_stats_completion_estimate() {
        let mut stats = WorkerStats::new(1);
        stats.smoothed_rate = 50.0 * 1024.0 * 1024.0; // 50 MB/s

        let remaining = 100 * 1024 * 1024; // 100 MB
        let eta = stats.estimate_completion_time(remaining, Duration::from_millis(200));

        // 100MB / 50MB/s = 2.0s + 0.2s overhead = 2.2s
        assert!((eta.as_secs_f64() - 2.2).abs() < 0.01);
    }

    #[test]
    fn test_segment_state_progress() {
        let mut segment = Segment::new(1, ByteRange::new(0, 999));
        assert_eq!(segment.remaining_bytes(), 1000);
        assert_eq!(segment.current_write_offset(), 0);
        assert_eq!(segment.progress_fraction(), 0.0);
        assert!(!segment.is_completed());

        segment.downloaded = 500;
        assert_eq!(segment.remaining_bytes(), 500);
        assert_eq!(segment.current_write_offset(), 500);
        assert_eq!(segment.progress_fraction(), 0.5);

        segment.downloaded = 1000;
        assert_eq!(segment.remaining_bytes(), 0);
        assert!(segment.is_completed());
    }
}
