use crate::network_model::NetworkModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilityWeights {
    pub throughput_weight: f64,
    pub rtt_inflation_weight: f64,
    pub retry_penalty_weight: f64,
    pub disk_pressure_weight: f64,
}

impl Default for UtilityWeights {
    fn default() -> Self {
        Self {
            throughput_weight: 1.0,
            rtt_inflation_weight: 0.6,
            retry_penalty_weight: 1.5,
            disk_pressure_weight: 2.0,
        }
    }
}

/// Evaluates connection and concurrency efficiency using multi-objective utility functions.
#[derive(Debug, Clone)]
pub struct UtilityController {
    weights: UtilityWeights,
    peak_throughput: f64,
}

impl UtilityController {
    pub fn new(weights: UtilityWeights) -> Self {
        Self {
            weights,
            peak_throughput: 1024.0 * 1024.0, // 1 MB/s initial baseline
        }
    }

    pub fn update_peak_throughput(&mut self, rate: f64) {
        if rate > self.peak_throughput {
            self.peak_throughput = rate;
        }
    }

    /// Computes overall utility score: higher is better.
    pub fn compute_utility(&self, model: &NetworkModel, is_disk_pressured: bool) -> f64 {
        let throughput_term = if self.peak_throughput > 0.0 {
            (model.bandwidth_estimate / self.peak_throughput).clamp(0.0, 1.5)
        } else {
            0.0
        };

        let rtt_penalty = model.rtt_inflation.clamp(0.0, 5.0);
        let retry_penalty = model.retry_rate.clamp(0.0, 1.0);
        let disk_penalty = if is_disk_pressured { 1.0 } else { 0.0 };

        (self.weights.throughput_weight * throughput_term)
            - (self.weights.rtt_inflation_weight * rtt_penalty)
            - (self.weights.retry_penalty_weight * retry_penalty)
            - (self.weights.disk_pressure_weight * disk_penalty)
    }

    /// Returns true if scaling from previous state to current state was beneficial.
    pub fn is_scaling_beneficial(&self, prev_utility: f64, curr_utility: f64, min_gain: f64) -> bool {
        curr_utility > prev_utility + min_gain
    }
}

impl Default for UtilityController {
    fn default() -> Self {
        Self::new(UtilityWeights::default())
    }
}
