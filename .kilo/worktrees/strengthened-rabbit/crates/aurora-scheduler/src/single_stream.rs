use aurora_core::traits::{Scheduler, SchedulerAction};
use aurora_core::types::{ByteRange, DownloadSnapshot, SegmentState};

/// Scheduler A: Baseline single-stream downloader (1 worker, 1 unbroken range).
#[derive(Debug, Default)]
pub struct SingleStreamScheduler {
    started: bool,
}

impl SingleStreamScheduler {
    pub fn new() -> Self {
        Self { started: false }
    }
}

impl Scheduler for SingleStreamScheduler {
    fn next_action(&mut self, state: &DownloadSnapshot) -> SchedulerAction {
        if self.started {
            return SchedulerAction::DoNothing;
        }

        // Check if there are any pending segments
        if let Some(first_pending) = state.segments.iter().find(|s| s.state == SegmentState::Pending) {
            self.started = true;
            return SchedulerAction::StartSegment {
                worker_id: 1,
                range: first_pending.range,
            };
        }

        // If no segments exist yet and file size is known, create full range [0, len - 1]
        if state.segments.is_empty() {
            if let Some(total) = state.total_bytes {
                if total > 0 {
                    self.started = true;
                    return SchedulerAction::StartSegment {
                        worker_id: 1,
                        range: ByteRange::new(0, total - 1),
                    };
                }
            }
        }

        SchedulerAction::DoNothing
    }

    fn reset(&mut self) {
        self.started = false;
    }
}
