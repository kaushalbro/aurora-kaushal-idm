use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Comprehensive snapshot of the network state for adaptive controllers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkModel {
    pub bandwidth_estimate: f64, // bytes/sec
    pub min_rtt: Duration,
    pub current_rtt: Duration,
    pub rtt_inflation: f64,
    pub rtt_variance_ms: f64,
    pub retry_rate: f64,
    pub connection_count: usize,
}

impl NetworkModel {
    pub fn new() -> Self {
        Self {
            bandwidth_estimate: 0.0,
            min_rtt: Duration::from_millis(50),
            current_rtt: Duration::from_millis(50),
            rtt_inflation: 0.0,
            rtt_variance_ms: 0.0,
            retry_rate: 0.0,
            connection_count: 1,
        }
    }
}

impl Default for NetworkModel {
    fn default() -> Self {
        Self::new()
    }
}
