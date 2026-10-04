use aurora_core::traits::{Scheduler, SchedulerAction};
use aurora_core::types::{ByteRange, DownloadSnapshot, SegmentState};

/// Scheduler B: Fixed static segmentation into N predetermined segments.
#[derive(Debug)]
pub struct FixedSegmentScheduler {
    connection_count: usize,
    initialized: bool,
}

impl FixedSegmentScheduler {
    pub fn new(connection_count: usize) -> Self {
        Self {
            connection_count: connection_count.max(1),
            initialized: false,
        }
    }

    pub fn connection_count(&self) -> usize {
        self.connection_count
    }

    /// Generates N fixed static byte ranges partitioning total_bytes.
    pub fn partition_fixed_ranges(total_bytes: u64, parts: usize) -> Vec<ByteRange> {
        if total_bytes == 0 {
            return Vec::new();
        }
        let parts = (parts.max(1) as u64).min(total_bytes);
        let chunk_size = total_bytes / parts;
        let mut ranges = Vec::with_capacity(parts as usize);

        for i in 0..parts {
            let start = i * chunk_size;
            let end = if i == parts - 1 {
                total_bytes - 1
            } else {
                (i + 1) * chunk_size - 1
            };
            if start <= end {
                ranges.push(ByteRange::new(start, end));
            }
        }
        ranges
    }
}

impl Scheduler for FixedSegmentScheduler {
    fn next_action(&mut self, state: &DownloadSnapshot) -> SchedulerAction {
        // Find any pending segment that has no worker assigned
        if let Some(pending) = state.segments.iter().find(|s| s.state == SegmentState::Pending) {
            let next_worker_id = (state.active_connections as u32) + 1;
            return SchedulerAction::StartSegment {
                worker_id: next_worker_id,
                range: pending.range,
            };
        }

        SchedulerAction::DoNothing
    }

    fn reset(&mut self) {
        self.initialized = false;
    }
}

#[cfg(test)]
mod partition_tests {
    use super::*;

    #[test]
    fn partitions_empty_and_tiny_files_without_underflow_or_overlap() {
        assert!(FixedSegmentScheduler::partition_fixed_ranges(0, 8).is_empty());
        for total in 1..33 {
            for connections in [0, 1, 8, 32] {
                let ranges = FixedSegmentScheduler::partition_fixed_ranges(total, connections);
                let mut offset = 0;
                for range in ranges {
                    assert_eq!(range.start, offset);
                    assert!(range.end >= range.start);
                    offset = range.end + 1;
                }
                assert_eq!(offset, total);
            }
        }
    }
}
