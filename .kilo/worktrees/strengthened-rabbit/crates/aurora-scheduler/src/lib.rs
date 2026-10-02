pub mod aurora_ect;
pub mod concurrency;
#[cfg(not(target_arch = "wasm32"))]
pub mod engine;
pub mod fixed;
pub mod largest_segment;
pub mod single_stream;

pub use aurora_ect::AuroraEctScheduler;
pub use concurrency::{AdaptiveConcurrencyController, ConcurrencyDecision};
#[cfg(not(target_arch = "wasm32"))]
pub use engine::DownloadEngine;
pub use fixed::FixedSegmentScheduler;
pub use largest_segment::LargestSegmentScheduler;
pub use single_stream::SingleStreamScheduler;

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::config::EngineConfig;
    use aurora_core::traits::Scheduler;
    use aurora_core::types::{
        ByteRange, DownloadId, DownloadSnapshot, DownloadStatus, Segment, SegmentState,
    };
    use std::path::PathBuf;
    use url::Url;

    #[test]
    fn test_fixed_partitioning_exact_coverage() {
        let total = 1000u64;
        let ranges = FixedSegmentScheduler::partition_fixed_ranges(total, 4);
        assert_eq!(ranges.len(), 4);

        assert_eq!(ranges[0], ByteRange::new(0, 249));
        assert_eq!(ranges[1], ByteRange::new(250, 499));
        assert_eq!(ranges[2], ByteRange::new(500, 749));
        assert_eq!(ranges[3], ByteRange::new(750, 999));

        let sum: u64 = ranges.iter().map(|r| r.len()).sum();
        assert_eq!(sum, total);
    }

    #[test]
    fn test_largest_segment_split() {
        let mut scheduler = LargestSegmentScheduler::new(100);

        let mut seg1 = Segment::new(1, ByteRange::new(0, 199));
        seg1.state = SegmentState::Downloading;
        seg1.downloaded = 100; // 100 remaining

        let mut seg2 = Segment::new(2, ByteRange::new(200, 999));
        seg2.state = SegmentState::Downloading;
        seg2.downloaded = 0; // 800 remaining (largest!)

        let snapshot = DownloadSnapshot {
            id: DownloadId::new(),
            url: Url::parse("https://example.com/test.bin").unwrap(),
            filename: "test.bin".to_string(),
            destination_path: PathBuf::from("test.bin"),
            total_bytes: Some(1000),
            downloaded_bytes: 100,
            status: DownloadStatus::Downloading,
            aggregate_rate: 1000.0,
            eta: None,
            active_connections: 2,
            segments: vec![seg1, seg2],
            server_capabilities: None,
        };

        let action = scheduler.next_action(&snapshot);
        match action {
            aurora_core::traits::SchedulerAction::SplitAndAssign {
                target_segment_id,
                new_range_for_existing,
                stolen_range_for_new_worker,
                ..
            } => {
                assert_eq!(target_segment_id, 2);
                assert_eq!(new_range_for_existing, ByteRange::new(200, 599));
                assert_eq!(stolen_range_for_new_worker, ByteRange::new(600, 999));
            }
            other => panic!("Expected SplitAndAssign, got {:?}", other),
        }
    }

    #[test]
    fn test_aurora_ect_rate_proportional_split() {
        let mut config = EngineConfig::default();
        config.min_segment_size = 10;
        let mut scheduler = AuroraEctScheduler::new(&config);

        // Worker 1: 10 MB/s (Slow straggler)
        scheduler.update_worker_rate(1, 10.0 * 1024.0 * 1024.0);
        // Idle Worker 2: 30 MB/s (Fast incoming worker)
        scheduler.update_worker_rate(2, 30.0 * 1024.0 * 1024.0);

        let mut seg1 = Segment::new(1, ByteRange::new(0, 999_999));
        seg1.state = SegmentState::Downloading;
        seg1.worker_id = Some(1);
        seg1.downloaded = 0; // 1,000,000 bytes remaining

        let snapshot = DownloadSnapshot {
            id: DownloadId::new(),
            url: Url::parse("https://example.com/test.bin").unwrap(),
            filename: "test.bin".to_string(),
            destination_path: PathBuf::from("test.bin"),
            total_bytes: Some(1_000_000),
            downloaded_bytes: 0,
            status: DownloadStatus::Downloading,
            aggregate_rate: 10.0 * 1024.0 * 1024.0,
            eta: None,
            active_connections: 1,
            segments: vec![seg1],
            server_capabilities: None,
        };

        let action = scheduler.next_action(&snapshot);
        match action {
            aurora_core::traits::SchedulerAction::SplitAndAssign {
                target_segment_id,
                new_range_for_existing,
                stolen_range_for_new_worker,
                ..
            } => {
                assert_eq!(target_segment_id, 1);
                // Worker 1 gets 25% (250,000 bytes), Worker 2 gets 75% (750,000 bytes)
                assert_eq!(new_range_for_existing.len(), 250_000);
                assert_eq!(stolen_range_for_new_worker.len(), 750_000);
                assert_eq!(
                    new_range_for_existing.len() + stolen_range_for_new_worker.len(),
                    1_000_000
                );
            }
            other => panic!("Expected SplitAndAssign with rate-proportional ratio, got {:?}", other),
        }
    }
}
