use aurora_core::config::EngineConfig;
use aurora_core::traits::{Scheduler, SchedulerAction};
use aurora_core::types::{ByteRange, DownloadSnapshot, Segment, SegmentState, SegmentId, WorkerId};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::info;

/// Scheduler D: AURORA Expected Completion Time (ECT) Scheduler.
/// Incorporates:
/// - ECT estimation: argmax(ECT)
/// - Rate-proportional segment splitting: Ra / (Ra + Rb)
/// - Adaptive segment duration sizing: clamp(rate * target_duration, min, max)
/// - Tail optimization & work stealing
#[derive(Debug)]
pub struct AuroraEctScheduler {
    max_connections: usize,
    target_segment_duration: Duration,
    min_segment_size: u64,
    max_segment_size: u64,
    setup_overhead: Duration,
    tail_hedging: bool,
    tail_threshold_pct: f64,
    max_redundant_bytes: u64,
    worker_rates: HashMap<WorkerId, f64>,
    hedged_source_segments: HashSet<SegmentId>,
}

impl AuroraEctScheduler {
    pub fn new(config: &EngineConfig) -> Self {
        Self {
            max_connections: config.max_connections.max(1),
            target_segment_duration: config.target_segment_duration,
            min_segment_size: config.min_segment_size,
            max_segment_size: config.max_segment_size,
            setup_overhead: Duration::from_millis(50),
            tail_hedging: config.tail_hedging,
            tail_threshold_pct: config.tail_threshold_pct,
            max_redundant_bytes: config.max_redundant_bytes,
            worker_rates: HashMap::new(),
            hedged_source_segments: HashSet::new(),
        }
    }

    /// Records or updates worker smoothed transfer speed (in bytes/sec).
    pub fn update_worker_rate(&mut self, worker_id: WorkerId, rate: f64) {
        if rate > 0.0 {
            self.worker_rates.insert(worker_id, rate);
        }
    }

    /// Retrieves estimated worker rate, with fallback to global average.
    pub fn get_worker_rate(&self, worker_id: Option<WorkerId>, default_rate: f64) -> f64 {
        if let Some(id) = worker_id {
            if let Some(&rate) = self.worker_rates.get(&id) {
                if rate > 1024.0 {
                    return rate;
                }
            }
        }
        if default_rate > 1024.0 {
            default_rate
        } else {
            100_000.0 // Conservative baseline 100 KB/s
        }
    }

    /// Computes target segment size for a given worker rate: clamp(rate * target_duration, min, max).
    pub fn calculate_adaptive_segment_size(&self, rate: f64) -> u64 {
        let size = (rate * self.target_segment_duration.as_secs_f64()).round() as u64;
        size.clamp(self.min_segment_size, self.max_segment_size)
    }

    /// Computes Expected Completion Time: ECT = remaining_bytes / rate + setup_overhead.
    pub fn compute_ect(&self, remaining_bytes: u64, worker_rate: f64) -> Duration {
        if remaining_bytes == 0 {
            return Duration::ZERO;
        }
        let transfer_time = remaining_bytes as f64 / worker_rate.max(1024.0);
        Duration::from_secs_f64(transfer_time) + self.setup_overhead
    }
}

impl Scheduler for AuroraEctScheduler {
    fn next_action(&mut self, state: &DownloadSnapshot) -> SchedulerAction {
        // Step 1: Dispatch any pending unallocated segment
        if let Some(pending) = state.segments.iter().find(|s| s.state == SegmentState::Pending) {
            if state.active_connections <= self.max_connections {
                let next_worker_id = (state.active_connections as u32).max(1);
                return SchedulerAction::StartSegment {
                    worker_id: next_worker_id,
                    range: pending.range,
                };
            }
        }

        // If active connections reached max, do not spawn additional workers
        if state.active_connections >= self.max_connections {
            return SchedulerAction::DoNothing;
        }

        // Determine effective minimum segment size for Endgame Mode vs normal phase
        let progress = if let Some(total) = state.total_bytes {
            if total > 0 {
                state.downloaded_bytes as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        };

        // BitTorrent Endgame Mode: When >= 90% downloaded, allow micro-splitting down to 64 KB
        // so all idle workers keep broadband fully saturated until the last byte!
        let effective_min_segment = if progress >= 0.90 {
            64 * 1024 // 64 KB endgame threshold
        } else {
            self.min_segment_size
        };

        // Step 2: Evaluate Expected Completion Time (ECT) across all active downloading segments
        let default_rate = state.aggregate_rate / state.active_connections.max(1) as f64;
        let mut max_ect = Duration::ZERO;
        let mut worst_straggler: Option<(&Segment, f64)> = None;

        for seg in &state.segments {
            if seg.state == SegmentState::Downloading {
                let remaining = seg.remaining_bytes();
                if remaining >= 2 * effective_min_segment {
                    let rate_a = self.get_worker_rate(seg.worker_id, default_rate);
                    let ect = self.compute_ect(remaining, rate_a);

                    if ect > max_ect {
                        max_ect = ect;
                        worst_straggler = Some((seg, rate_a));
                    }
                }
            }
        }

        // Step 3: If a straggler is detected with remaining time exceeding setup overhead, split work
        if let Some((straggler_seg, rate_a)) = worst_straggler {
            if max_ect > self.setup_overhead {
                let idle_worker_id = (state.active_connections as u32) + 1;
                let rate_b = self.get_worker_rate(Some(idle_worker_id), default_rate);

                // Rate-proportional split ratio:
                // Portion A: Ra / (Ra + Rb)
                // Portion B: Rb / (Ra + Rb)
                let total_rate = rate_a + rate_b;
                let left_ratio = (rate_a / total_rate).clamp(0.15, 0.85);

                let current_offset = straggler_seg.current_write_offset();
                let remaining_range = ByteRange::new(current_offset, straggler_seg.range.end);

                if let Some((left_rem, right_stolen)) = remaining_range.split_ratio(left_ratio) {
                    if left_rem.len() >= effective_min_segment && right_stolen.len() >= effective_min_segment {
                        let new_existing_range =
                            ByteRange::new(straggler_seg.range.start, left_rem.end);

                        info!(
                            "AURORA-ECT: Straggler segment {} (ECT: {:.2?}s, rate: {:.1}MB/s) split proportionally ({:.0}% / {:.0}%) -> stolen range {} assigned to worker {}",
                            straggler_seg.id,
                            max_ect.as_secs_f64(),
                            rate_a / (1024.0 * 1024.0),
                            left_ratio * 100.0,
                            (1.0 - left_ratio) * 100.0,
                            right_stolen,
                            idle_worker_id
                        );

                        return SchedulerAction::SplitAndAssign {
                            target_segment_id: straggler_seg.id,
                            new_range_for_existing: new_existing_range,
                            stolen_range_for_new_worker: right_stolen,
                            idle_worker_id,
                        };
                    }
                }
            }
        }

        // Step 4: Tail optimization & speculative hedging (Hedged Requests for extreme tail stragglers)
        if self.tail_hedging && progress >= self.tail_threshold_pct {
            for seg in &state.segments {
                if seg.state == SegmentState::Downloading && !self.hedged_source_segments.contains(&seg.id) {
                    let rem = seg.remaining_bytes();
                    if rem > 0 && rem <= self.max_redundant_bytes {
                        let rate = self.get_worker_rate(seg.worker_id, default_rate);
                        let ect = self.compute_ect(rem, rate);
                        if ect > Duration::from_millis(500) {
                            let hedge_worker_id = (state.active_connections as u32) + 1;
                            let current_offset = seg.current_write_offset();
                            let hedge_range = ByteRange::new(current_offset, seg.range.end);

                            info!(
                                "AURORA-ECT: Speculatively hedging tail straggler segment {} (remaining: {} bytes, ECT: {:.2?}s)",
                                seg.id, rem, ect.as_secs_f64()
                            );

                            self.hedged_source_segments.insert(seg.id);

                            return SchedulerAction::DuplicateTail {
                                source_segment_id: seg.id,
                                hedge_range,
                                worker_id: hedge_worker_id,
                            };
                        }
                    }
                }
            }
        }

        SchedulerAction::DoNothing
    }

    fn reset(&mut self) {
        self.worker_rates.clear();
        self.hedged_source_segments.clear();
    }
}
