use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Historical performance profile for an observed remote host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostProfile {
    pub host: String,
    pub preferred_connection_count: usize,
    pub average_throughput: f64,
    pub average_rtt_ms: f64,
    pub range_reliability: f64, // 0.0 to 1.0
    pub preferred_protocol: String,
    pub sample_count: u32,
    pub updated_at: DateTime<Utc>,
}

impl HostProfile {
    pub fn new(host: String) -> Self {
        Self {
            host,
            preferred_connection_count: 4,
            average_throughput: 0.0,
            average_rtt_ms: 50.0,
            range_reliability: 1.0,
            preferred_protocol: "HTTP/2".to_string(),
            sample_count: 0,
            updated_at: Utc::now(),
        }
    }

    /// Incorporates a new observation and updates moving statistics.
    pub fn record_observation(&mut self, throughput: f64, rtt: Duration, ranges_supported: bool) {
        self.sample_count += 1;
        self.updated_at = Utc::now();

        let rtt_ms = rtt.as_secs_f64() * 1000.0;
        let alpha = 0.3; // Weight for new observation

        if self.sample_count == 1 {
            self.average_throughput = throughput;
            self.average_rtt_ms = rtt_ms;
            self.range_reliability = if ranges_supported { 1.0 } else { 0.0 };
        } else {
            self.average_throughput = (1.0 - alpha) * self.average_throughput + alpha * throughput;
            self.average_rtt_ms = (1.0 - alpha) * self.average_rtt_ms + alpha * rtt_ms;
            let range_val = if ranges_supported { 1.0 } else { 0.0 };
            self.range_reliability = (1.0 - alpha) * self.range_reliability + alpha * range_val;
        }
    }

    /// Decays historical data weight when profiles become stale.
    pub fn apply_time_decay(&mut self, half_life_days: f64) {
        let elapsed_secs = (Utc::now() - self.updated_at).num_seconds().max(0) as f64;
        let half_life_secs = half_life_days * 86400.0;
        let decay = (-elapsed_secs / half_life_secs * 0.693).exp();

        self.sample_count = (self.sample_count as f64 * decay).round() as u32;
    }
}
