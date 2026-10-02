use aurora_core::traits::RateEstimator;
use std::collections::VecDeque;
use std::time::Duration;

/// Industrial-Grade Sliding Window + EWMA Throughput Estimator.
/// Combines an O(1) 1.2-second sliding-window ring buffer (which absorbs sub-second TCP burstiness/jitter)
/// with a continuous-time EWMA filter to output stable, physically accurate live speeds.
#[derive(Debug, Clone)]
pub struct EwmaRateEstimator {
    tau_secs: f64,
    window_duration: Duration,
    samples: VecDeque<(u64, Duration)>,
    window_bytes: u64,
    window_dt: Duration,
    smoothed_rate: f64,
    instantaneous_rate: f64,
    total_bytes: u64,
    total_duration: Duration,
    initialized: bool,
}

impl EwmaRateEstimator {
    pub fn new(tau_secs: f64) -> Self {
        Self {
            tau_secs: tau_secs.max(0.1),
            window_duration: Duration::from_millis(1200), // 1.2s sliding window absorbs TCP packet jitter
            samples: VecDeque::with_capacity(32),
            window_bytes: 0,
            window_dt: Duration::ZERO,
            smoothed_rate: 0.0,
            instantaneous_rate: 0.0,
            total_bytes: 0,
            total_duration: Duration::ZERO,
            initialized: false,
        }
    }

    pub fn with_default_tau() -> Self {
        Self::new(0.8)
    }

    pub fn average_rate(&self) -> f64 {
        let secs = self.total_duration.as_secs_f64();
        if secs > 0.0 {
            self.total_bytes as f64 / secs
        } else {
            0.0
        }
    }
}

impl Default for EwmaRateEstimator {
    fn default() -> Self {
        Self::with_default_tau()
    }
}

impl RateEstimator for EwmaRateEstimator {
    fn update(&mut self, bytes: u64, dt: Duration) {
        let dt_secs = dt.as_secs_f64();
        if dt_secs <= 0.0 {
            return;
        }

        self.total_bytes += bytes;
        self.total_duration += dt;

        // Push current sample to sliding window
        self.samples.push_back((bytes, dt));
        self.window_bytes += bytes;
        self.window_dt += dt;

        // Purge samples that push window_dt beyond window_duration
        while self.window_dt > self.window_duration && self.samples.len() > 2 {
            if let Some((old_b, old_dt)) = self.samples.pop_front() {
                self.window_bytes = self.window_bytes.saturating_sub(old_b);
                self.window_dt = self.window_dt.saturating_sub(old_dt);
            } else {
                break;
            }
        }

        // Calculate true sliding-window throughput
        let w_secs = self.window_dt.as_secs_f64();
        let window_rate = if w_secs > 0.01 {
            self.window_bytes as f64 / w_secs
        } else {
            bytes as f64 / dt_secs
        };

        self.instantaneous_rate = bytes as f64 / dt_secs;

        if !self.initialized {
            self.smoothed_rate = window_rate;
            self.initialized = true;
        } else {
            let alpha = (1.0 - (-dt_secs / self.tau_secs).exp()).clamp(0.05, 0.95);
            self.smoothed_rate = alpha * window_rate + (1.0 - alpha) * self.smoothed_rate;
        }
    }

    fn smoothed_rate(&self) -> f64 {
        self.smoothed_rate
    }

    fn instantaneous_rate(&self) -> f64 {
        self.instantaneous_rate
    }

    fn reset(&mut self) {
        self.smoothed_rate = 0.0;
        self.instantaneous_rate = 0.0;
        self.total_bytes = 0;
        self.total_duration = Duration::ZERO;
        self.samples.clear();
        self.window_bytes = 0;
        self.window_dt = Duration::ZERO;
        self.initialized = false;
    }
}
