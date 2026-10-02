use aurora_core::config::{EngineConfig, SchedulerType};
use aurora_core::traits::{RateEstimator, Scheduler};
use aurora_core::types::{
    ByteRange, DownloadId, DownloadSnapshot, DownloadStatus, Segment, SegmentState, WorkerId,
};
use aurora_metrics::ewma::EwmaRateEstimator;
use aurora_metrics::rtt::RttTracker;
use aurora_scheduler::aurora_ect::AuroraEctScheduler;
use aurora_scheduler::fixed::FixedSegmentScheduler;
use aurora_scheduler::largest_segment::LargestSegmentScheduler;
use aurora_scheduler::single_stream::SingleStreamScheduler;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct AuroraWasmEngine {
    url: Url,
    total_bytes: Option<u64>,
    #[allow(dead_code)]
    scheduler_type: SchedulerType,
    #[allow(dead_code)]
    connections: usize,
    scheduler: Box<dyn Scheduler>,
    ewma: EwmaRateEstimator,
    rtt_trackers: HashMap<WorkerId, RttTracker>,
    segments: Vec<Segment>,
    worker_rates: HashMap<WorkerId, f64>,
    downloaded_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmSegmentState {
    pub id: u32,
    pub start: u64,
    pub end: u64,
    pub downloaded: u64,
    pub state: String,
    pub worker_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmAction {
    pub action_type: String, // "start_segment", "split_and_assign", "do_nothing"
    pub segment_id: Option<u32>,
    pub worker_id: Option<u32>,
    pub start: Option<u64>,
    pub end: Option<u64>,
    pub target_segment_id: Option<u32>,
    pub new_start: Option<u64>,
    pub new_end: Option<u64>,
    pub stolen_start: Option<u64>,
    pub stolen_end: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmEngineSnapshot {
    pub url: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub progress_pct: f64,
    pub smoothed_speed_bytes_per_sec: f64,
    pub smoothed_speed_mbps: f64,
    pub eta_seconds: Option<f64>,
    pub active_connections: usize,
    pub segments: Vec<WasmSegmentState>,
}

#[wasm_bindgen]
impl AuroraWasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(
        url_str: &str,
        content_length: Option<u64>,
        connections: usize,
        scheduler_type_str: &str,
    ) -> Result<AuroraWasmEngine, JsValue> {
        let url = Url::parse(url_str).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let conns = connections.clamp(1, 32);

        let sched_type = match scheduler_type_str.to_lowercase().trim() {
            "single" | "single-stream" => SchedulerType::SingleStream,
            "fixed" => SchedulerType::Fixed,
            "largest-segment" | "largest" => SchedulerType::LargestSegment,
            _ => SchedulerType::AuroraEct,
        };

        let mut config = EngineConfig::default();
        config.scheduler_type = sched_type;
        config.initial_connections = conns;
        config.max_connections = conns;

        let scheduler: Box<dyn Scheduler> = match sched_type {
            SchedulerType::SingleStream => Box::new(SingleStreamScheduler::new()),
            SchedulerType::Fixed => Box::new(FixedSegmentScheduler::new(conns)),
            SchedulerType::LargestSegment => {
                Box::new(LargestSegmentScheduler::new(config.min_segment_size))
            }
            SchedulerType::AuroraEct => Box::new(AuroraEctScheduler::new(&config)),
        };

        let mut segments = Vec::new();
        if let Some(total) = content_length {
            if total > 0 {
                if conns > 1 {
                    let ranges = FixedSegmentScheduler::partition_fixed_ranges(total, conns);
                    for (i, range) in ranges.into_iter().enumerate() {
                        segments.push(Segment::new((i + 1) as u32, range));
                    }
                } else {
                    segments.push(Segment::new(1, ByteRange::new(0, total - 1)));
                }
            }
        }

        Ok(Self {
            url,
            total_bytes: content_length,
            scheduler_type: sched_type,
            connections: conns,
            scheduler,
            ewma: EwmaRateEstimator::with_default_tau(),
            rtt_trackers: HashMap::new(),
            segments,
            worker_rates: HashMap::new(),
            downloaded_bytes: 0,
        })
    }

    /// Evaluates current download state and returns the next scheduling action.
    pub fn get_next_action(&mut self, active_conns: usize) -> Result<JsValue, JsValue> {
        let snapshot = DownloadSnapshot {
            id: DownloadId::new(),
            url: self.url.clone(),
            filename: "file".to_string(),
            destination_path: std::path::PathBuf::from("file"),
            total_bytes: self.total_bytes,
            downloaded_bytes: self.downloaded_bytes,
            status: DownloadStatus::Downloading,
            aggregate_rate: self.ewma.smoothed_rate(),
            eta: None,
            active_connections: active_conns,
            segments: self.segments.clone(),
            server_capabilities: None,
        };

        let action = self.scheduler.next_action(&snapshot);

        let wasm_action = match action {
            aurora_core::traits::SchedulerAction::DoNothing => WasmAction {
                action_type: "do_nothing".to_string(),
                segment_id: None,
                worker_id: None,
                start: None,
                end: None,
                target_segment_id: None,
                new_start: None,
                new_end: None,
                stolen_start: None,
                stolen_end: None,
            },
            aurora_core::traits::SchedulerAction::StartSegment { worker_id, range } => {
                let seg_id = if let Some(s) = self.segments.iter_mut().find(|s| s.range == range) {
                    s.state = SegmentState::Downloading;
                    s.worker_id = Some(worker_id);
                    s.id
                } else {
                    let next_id = self.segments.len() as u32 + 1;
                    let mut s = Segment::new(next_id, range);
                    s.state = SegmentState::Downloading;
                    s.worker_id = Some(worker_id);
                    self.segments.push(s);
                    next_id
                };
                WasmAction {
                    action_type: "start_segment".to_string(),
                    segment_id: Some(seg_id),
                    worker_id: Some(worker_id),
                    start: Some(range.start),
                    end: Some(range.end),
                    target_segment_id: None,
                    new_start: None,
                    new_end: None,
                    stolen_start: None,
                    stolen_end: None,
                }
            }
            aurora_core::traits::SchedulerAction::SplitAndAssign {
                target_segment_id,
                new_range_for_existing,
                stolen_range_for_new_worker,
                idle_worker_id,
            } => {
                if let Some(target) = self.segments.iter_mut().find(|s| s.id == target_segment_id) {
                    target.range = new_range_for_existing;
                }
                let new_id = self.segments.len() as u32 + 1;
                let mut new_seg = Segment::new(new_id, stolen_range_for_new_worker);
                new_seg.worker_id = Some(idle_worker_id);
                new_seg.state = SegmentState::Downloading;
                self.segments.push(new_seg);

                WasmAction {
                    action_type: "split_and_assign".to_string(),
                    segment_id: Some(new_id),
                    worker_id: Some(idle_worker_id),
                    start: Some(stolen_range_for_new_worker.start),
                    end: Some(stolen_range_for_new_worker.end),
                    target_segment_id: Some(target_segment_id),
                    new_start: Some(new_range_for_existing.start),
                    new_end: Some(new_range_for_existing.end),
                    stolen_start: Some(stolen_range_for_new_worker.start),
                    stolen_end: Some(stolen_range_for_new_worker.end),
                }
            }
            _ => WasmAction {
                action_type: "do_nothing".to_string(),
                segment_id: None,
                worker_id: None,
                start: None,
                end: None,
                target_segment_id: None,
                new_start: None,
                new_end: None,
                stolen_start: None,
                stolen_end: None,
            },
        };

        serde_wasm_bindgen::to_value(&wasm_action).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Records chunk progress from a worker.
    pub fn record_progress(
        &mut self,
        segment_id: u32,
        worker_id: u32,
        chunk_bytes: u64,
        duration_ms: f64,
    ) {
        self.downloaded_bytes += chunk_bytes;
        let dt = Duration::from_secs_f64((duration_ms / 1000.0).max(0.0001));
        self.ewma.update(chunk_bytes, dt);

        let rtt_tracker = self
            .rtt_trackers
            .entry(worker_id)
            .or_insert_with(RttTracker::new);
        rtt_tracker.record_sample(dt);

        let rate = chunk_bytes as f64 / (duration_ms / 1000.0).max(0.0001);
        self.worker_rates.insert(worker_id, rate);

        if let Some(seg) = self.segments.iter_mut().find(|s| s.id == segment_id) {
            seg.downloaded += chunk_bytes;
            seg.worker_id = Some(worker_id);
            seg.state = SegmentState::Downloading;
            if seg.downloaded >= seg.range.len() {
                seg.state = SegmentState::Completed;
            }
        }
    }

    /// Marks a segment as completed.
    pub fn mark_segment_completed(&mut self, segment_id: u32) {
        if let Some(seg) = self.segments.iter_mut().find(|s| s.id == segment_id) {
            seg.state = SegmentState::Completed;
            seg.downloaded = seg.range.len();
        }
    }

    /// Returns a full snapshot of the current download state for the UI.
    pub fn get_snapshot(&self) -> Result<JsValue, JsValue> {
        let speed = self.ewma.smoothed_rate();
        let speed_mbps = (speed * 8.0) / (1024.0 * 1024.0);

        let progress_pct = if let Some(total) = self.total_bytes {
            if total > 0 {
                (self.downloaded_bytes as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let eta_seconds = if let Some(total) = self.total_bytes {
            if speed > 1024.0 && total > self.downloaded_bytes {
                Some((total - self.downloaded_bytes) as f64 / speed)
            } else {
                None
            }
        } else {
            None
        };

        let active_conns = self
            .segments
            .iter()
            .filter(|s| s.state == SegmentState::Downloading)
            .count();

        let wasm_segments = self
            .segments
            .iter()
            .map(|s| WasmSegmentState {
                id: s.id,
                start: s.range.start,
                end: s.range.end,
                downloaded: s.downloaded,
                state: format!("{:?}", s.state).to_lowercase(),
                worker_id: s.worker_id,
            })
            .collect();

        let snapshot = WasmEngineSnapshot {
            url: self.url.to_string(),
            total_bytes: self.total_bytes,
            downloaded_bytes: self.downloaded_bytes,
            progress_pct,
            smoothed_speed_bytes_per_sec: speed,
            smoothed_speed_mbps: speed_mbps,
            eta_seconds,
            active_connections: active_conns,
            segments: wasm_segments,
        };

        serde_wasm_bindgen::to_value(&snapshot).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Computes a fast SHA-256 checksum of an in-memory byte buffer.
#[wasm_bindgen]
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Computes a fast BLAKE3 checksum of an in-memory byte buffer.
#[wasm_bindgen]
pub fn compute_blake3(data: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    hasher.finalize().to_hex().to_string()
}

/// Verifies whether data matches the expected checksum string using SHA256 or BLAKE3.
#[wasm_bindgen]
pub fn verify_checksum(data: &[u8], expected_hex: &str, algorithm: &str) -> bool {
    let actual = match algorithm.to_lowercase().trim() {
        "blake3" => compute_blake3(data),
        _ => compute_sha256(data),
    };
    actual.eq_ignore_ascii_case(expected_hex.trim())
}

/// Returns the AURORA engine build version string.
#[wasm_bindgen]
pub fn aurora_version() -> String {
    "AURORA Kaushal IDM v0.3.0 (WebAssembly Core)".to_string()
}
