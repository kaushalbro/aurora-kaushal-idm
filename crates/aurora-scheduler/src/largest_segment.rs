use aurora_core::traits::{Scheduler, SchedulerAction};
use aurora_core::types::{ByteRange, DownloadSnapshot, Segment, SegmentState};
use tracing::debug;

/// Scheduler C: Largest-Segment Dynamic Splitting (IDM-style baseline).
/// When a worker becomes idle, it finds the active segment with the largest remaining bytes
/// and splits it in half (50/50).
#[derive(Debug)]
pub struct LargestSegmentScheduler {
    min_split_size: u64,
}

impl LargestSegmentScheduler {
    pub fn new(min_split_size: u64) -> Self {
        Self {
            min_split_size: min_split_size.max(2),
        }
    }
}

impl Default for LargestSegmentScheduler {
    fn default() -> Self {
        Self::new(1024 * 1024) // 1 MB default minimum split
    }
}

impl Scheduler for LargestSegmentScheduler {
    fn next_action(&mut self, state: &DownloadSnapshot) -> SchedulerAction {
        // Step 1: Check for any pending unallocated segment
        if let Some(pending) = state.segments.iter().find(|s| s.state == SegmentState::Pending) {
            let next_worker_id = (state.active_connections as u32) + 1;
            return SchedulerAction::StartSegment {
                worker_id: next_worker_id,
                range: pending.range,
            };
        }

        // Step 2: If all existing segments are downloading or completed, find largest active segment
        let mut largest_segment: Option<&Segment> = None;
        let mut max_remaining = 0u64;

        for seg in &state.segments {
            if seg.state == SegmentState::Downloading {
                let remaining = seg.remaining_bytes();
                if remaining > max_remaining {
                    max_remaining = remaining;
                    largest_segment = Some(seg);
                }
            }
        }

        if let Some(target) = largest_segment {
            if max_remaining >= self.min_split_size * 2 {
                let current_offset = target.current_write_offset();
                let remaining_range = ByteRange::new(current_offset, target.range.end);

                // 50/50 equal split of the remaining portion
                if let Some((left_rem, right_stolen)) = remaining_range.split_ratio(0.5) {
                    let new_existing_range = ByteRange::new(target.range.start, left_rem.end);
                    let idle_worker_id = (state.active_connections as u32) + 1;

                    debug!(
                        "LargestSegmentScheduler: splitting segment {} (remaining {} bytes) 50/50 -> stolen range {}",
                        target.id, max_remaining, right_stolen
                    );

                    return SchedulerAction::SplitAndAssign {
                        target_segment_id: target.id,
                        new_range_for_existing: new_existing_range,
                        stolen_range_for_new_worker: right_stolen,
                        idle_worker_id,
                    };
                }
            }
        }

        SchedulerAction::DoNothing
    }

    fn reset(&mut self) {}
}
