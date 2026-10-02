use aurora_core::config::EngineConfig;
use aurora_core::events::EventSender;
use aurora_core::types::{DownloadId, Segment, SegmentState, ServerCapabilities};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentDto {
    pub index: usize,
    pub start_byte: u64,
    pub end_byte: u64,
    pub downloaded_bytes: u64,
    pub state: String,
    pub speed_bps: f64,
    pub stream_id: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItemDto {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub destination_path: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub current_speed: f64,
    pub peak_speed: f64,
    pub min_speed: f64,
    pub eta_secs: Option<u64>,
    pub status: String,
    pub active_connections: usize,
    pub http_version: String,
    pub etag: Option<String>,
    pub accepts_ranges: bool,
    pub is_active: bool,
    pub segments: Vec<SegmentDto>,
    pub start_time_str: String,
    pub end_time_str: Option<String>,
    pub elapsed_duration_secs: Option<u64>,
    pub server_rtt_ms: Option<u64>,
    pub server_name: Option<String>,
    pub health_rating: Option<String>,
    pub checksum_result: Option<String>,
}

pub struct InternalDownloadItem {
    pub id: DownloadId,
    pub url: String,
    pub filename: String,
    pub destination_path: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub current_speed: f64,
    pub peak_speed: f64,
    pub min_speed: f64,
    pub eta: Option<Duration>,
    pub status: String,
    pub active_connections: usize,
    pub http_version: String,
    pub etag: Option<String>,
    pub accepts_ranges: bool,
    pub is_active: bool,
    pub segments: Vec<Segment>,
    pub pause_signal: Arc<AtomicBool>,
    pub cancel_signal: Arc<AtomicBool>,
    pub start_time_str: String,
    pub start_instant: Instant,
    pub end_time_str: Option<String>,
    pub elapsed_duration: Option<Duration>,
    pub server_rtt_ms: Option<u64>,
    pub server_name: Option<String>,
    pub health_rating: Option<String>,
    pub checksum_result: Option<String>,
}

impl InternalDownloadItem {
    pub fn to_dto(&self) -> DownloadItemDto {
        let seg_dtos: Vec<SegmentDto> = self
            .segments
            .iter()
            .map(|s| SegmentDto {
                index: s.id as usize,
                start_byte: s.range.start,
                end_byte: s.range.end,
                downloaded_bytes: s.downloaded,
                state: match s.state {
                    SegmentState::Pending => "pending".to_string(),
                    SegmentState::Downloading => "downloading".to_string(),
                    SegmentState::Completed => "completed".to_string(),
                    SegmentState::Failed => "failed".to_string(),
                    SegmentState::Paused => "paused".to_string(),
                },
                speed_bps: 0.0,
                stream_id: s.worker_id.map(|w| w as usize),
            })
            .collect();

        DownloadItemDto {
            id: self.id.to_string(),
            url: self.url.clone(),
            filename: self.filename.clone(),
            destination_path: self.destination_path.clone(),
            total_bytes: self.total_bytes,
            downloaded_bytes: self.downloaded_bytes,
            current_speed: self.current_speed,
            peak_speed: self.peak_speed,
            min_speed: self.min_speed,
            eta_secs: self.eta.map(|d| d.as_secs()),
            status: self.status.clone(),
            active_connections: self.active_connections,
            http_version: self.http_version.clone(),
            etag: self.etag.clone(),
            accepts_ranges: self.accepts_ranges,
            is_active: self.is_active,
            segments: seg_dtos,
            start_time_str: self.start_time_str.clone(),
            end_time_str: self.end_time_str.clone(),
            elapsed_duration_secs: self.elapsed_duration.map(|d| d.as_secs()),
            server_rtt_ms: self.server_rtt_ms,
            server_name: self.server_name.clone(),
            health_rating: self.health_rating.clone(),
            checksum_result: self.checksum_result.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilitiesDto {
    pub url: String,
    pub content_length: Option<u64>,
    pub accepts_ranges: bool,
    pub etag: Option<String>,
    pub content_type: Option<String>,
    pub server_name: Option<String>,
    pub http_version: String,
    pub rtt_ms: Option<u64>,
    pub health_rating: Option<String>,
    pub suggested_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfigDto {
    pub download_dir: String,
    pub initial_connections: usize,
    pub max_connections: usize,
    pub chunk_size_bytes: usize,
    pub write_buffer_bytes: usize,
    pub request_timeout_secs: u64,
    pub max_retries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryPayload {
    pub total_speed_bps: f64,
    pub total_speed_str: String,
    pub active_tasks: usize,
    pub speed_history: Vec<f64>,
    pub downloads: Vec<DownloadItemDto>,
}

pub struct AppState {
    pub config: EngineConfig,
    pub downloads: Vec<InternalDownloadItem>,
    pub event_tx: EventSender,
    pub speed_history: Vec<f64>,
    pub last_speed_sample: Instant,
    pub probed_caps: HashMap<String, ServerCapabilities>,
}

impl AppState {
    pub fn new(event_tx: EventSender) -> Self {
        Self {
            config: EngineConfig::default(),
            downloads: Vec::new(),
            event_tx,
            speed_history: vec![0.0; 60],
            last_speed_sample: Instant::now(),
            probed_caps: HashMap::new(),
        }
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;
