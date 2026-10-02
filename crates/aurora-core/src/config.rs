use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Scheduling strategy for allocating segments and connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SchedulerType {
    /// Single stream (1 connection, baseline).
    SingleStream,
    /// Fixed static segmentation (e.g. 8 static equal segments).
    Fixed,
    /// Largest segment dynamic splitting (IDM-style baseline: splits largest active segment).
    LargestSegment,
    /// AURORA Expected Completion Time (ECT) scheduling with rate-proportional splitting.
    #[default]
    AuroraEct,
}

impl std::fmt::Display for SchedulerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingleStream => write!(f, "Single Stream (Baseline)"),
            Self::Fixed => write!(f, "Fixed Segmentation"),
            Self::LargestSegment => write!(f, "Largest-Segment Split (IDM-style)"),
            Self::AuroraEct => write!(f, "AURORA Expected Completion Time (ECT)"),
        }
    }
}

/// Comprehensive engine and download configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Default directory where downloaded files are placed.
    pub download_dir: PathBuf,
    /// Maximum simultaneous downloads.
    pub max_simultaneous_downloads: usize,
    /// Scheduler algorithm choice.
    pub scheduler_type: SchedulerType,
    /// Minimum connection count for multi-segment downloads.
    pub min_connections: usize,
    /// Maximum connection count for multi-segment downloads.
    pub max_connections: usize,
    /// Initial connection count to start with.
    pub initial_connections: usize,
    /// Target duration (in seconds) for each segment.
    pub target_segment_duration: Duration,
    /// Minimum allowed segment size in bytes (e.g., 1 MB).
    pub min_segment_size: u64,
    /// Maximum allowed segment size in bytes (e.g., 512 MB).
    pub max_segment_size: u64,
    /// Whether adaptive concurrency discovery is enabled.
    pub adaptive_concurrency: bool,
    /// Minimum marginal throughput gain required to increase worker count (e.g., 0.05 = 5%).
    pub min_marginal_gain: f64,
    /// RTT inflation threshold above min RTT to trigger throttling (e.g., 0.50 = 50%).
    pub rtt_inflation_threshold: f64,
    /// Maximum number of retry attempts for failed segments.
    pub max_retries: u32,
    /// Base backoff duration before retry.
    pub base_backoff: Duration,
    /// Maximum backoff duration before retry.
    pub max_backoff: Duration,
    /// Maximum in-memory writer queue size in bytes before applying backpressure.
    pub disk_queue_limit_bytes: usize,
    /// Periodic disk flush interval.
    pub disk_flush_interval: Duration,
    /// Journal state checkpoint interval.
    pub checkpoint_interval: Duration,
    /// Whether to enable speculative tail hedging in the final completion phase.
    pub tail_hedging: bool,
    /// Progress threshold at which tail hedging may activate (e.g., 0.99 = 99%).
    pub tail_threshold_pct: f64,
    /// Maximum redundant bytes allowed during tail hedging.
    pub max_redundant_bytes: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let download_dir = crate::platform::default_download_dir();
        Self {
            download_dir,
            max_simultaneous_downloads: 3,
            scheduler_type: SchedulerType::AuroraEct,
            min_connections: 1,
            max_connections: 32,
            initial_connections: 16,
            target_segment_duration: Duration::from_secs(3),
            min_segment_size: 128 * 1024,           // 128 KB (fast work-stealing)
            max_segment_size: 512 * 1024 * 1024,    // 512 MB
            adaptive_concurrency: true,
            min_marginal_gain: 0.05,
            rtt_inflation_threshold: 0.50,
            max_retries: 5,
            base_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            disk_queue_limit_bytes: 64 * 1024 * 1024, // 64 MB
            disk_flush_interval: Duration::from_secs(1),
            checkpoint_interval: Duration::from_secs(2),
            tail_hedging: true,
            tail_threshold_pct: 0.98,
            max_redundant_bytes: 8 * 1024 * 1024, // 8 MB max redundant bytes
        }
    }
}
