use std::time::Duration;

/// Real-time RTT tracking, baseline estimation, and queue delay/inflation detector.
#[derive(Debug, Clone)]
pub struct RttTracker {
    min_rtt: Duration,
    current_rtt: Duration,
    smoothed_rtt: Duration,
    rtt_variance_ms: f64,
    sample_count: u64,
}

impl RttTracker {
    pub fn new() -> Self {
        Self {
            min_rtt: Duration::from_millis(50),
            current_rtt: Duration::from_millis(50),
            smoothed_rtt: Duration::from_millis(50),
            rtt_variance_ms: 0.0,
            sample_count: 0,
        }
    }

    pub fn record_sample(&mut self, sample: Duration) {
        self.current_rtt = sample;
        self.sample_count += 1;

        if self.sample_count == 1 || sample < self.min_rtt {
            self.min_rtt = sample;
        }

        let sample_ms = sample.as_secs_f64() * 1000.0;
        let smoothed_ms = self.smoothed_rtt.as_secs_f64() * 1000.0;

        if self.sample_count == 1 {
            self.smoothed_rtt = sample;
            self.rtt_variance_ms = 0.0;
        } else {
            // Standard RFC 6298 Jacobson/Karels RTT estimator
            let diff = (sample_ms - smoothed_ms).abs();
            self.rtt_variance_ms = 0.75 * self.rtt_variance_ms + 0.25 * diff;
            let new_smoothed_ms = 0.875 * smoothed_ms + 0.125 * sample_ms;
            self.smoothed_rtt = Duration::from_secs_f64(new_smoothed_ms / 1000.0);
        }
    }

    /// Calculates queue delay inflation relative to min baseline: (current - min) / min
    pub fn rtt_inflation(&self) -> f64 {
        let min_ms = self.min_rtt.as_secs_f64();
        if min_ms <= 0.001 {
            return 0.0;
        }
        let curr_ms = self.smoothed_rtt.as_secs_f64();
        ((curr_ms - min_ms) / min_ms).max(0.0)
    }

    pub fn min_rtt(&self) -> Duration {
        self.min_rtt
    }

    pub fn current_rtt(&self) -> Duration {
        self.current_rtt
    }

    pub fn smoothed_rtt(&self) -> Duration {
        self.smoothed_rtt
    }

    pub fn variance_ms(&self) -> f64 {
        self.rtt_variance_ms
    }
}

impl Default for RttTracker {
    fn default() -> Self {
        Self::new()
    }
}
