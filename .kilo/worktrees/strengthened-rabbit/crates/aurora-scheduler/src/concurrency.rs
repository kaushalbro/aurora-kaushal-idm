use aurora_core::config::EngineConfig;
use aurora_metrics::network_model::NetworkModel;
use aurora_metrics::utility::UtilityController;
use std::time::{Duration, Instant};
use tracing::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrencyDecision {
    Increase,
    Decrease,
    Hold,
}

/// Dynamic concurrency controller that optimizes worker count based on marginal throughput gain,
/// RTT inflation, and error feedback.
#[derive(Debug)]
pub struct AdaptiveConcurrencyController {
    min_connections: usize,
    max_connections: usize,
    current_target_connections: usize,
    min_marginal_gain: f64,
    rtt_inflation_threshold: f64,
    control_interval: Duration,
    last_evaluation: Instant,
    last_bandwidth: f64,
    utility_controller: UtilityController,
    last_utility: f64,
    exploration_phase: bool,
}

impl AdaptiveConcurrencyController {
    pub fn new(config: &EngineConfig) -> Self {
        Self {
            min_connections: config.min_connections.max(1),
            max_connections: config.max_connections.max(1),
            current_target_connections: config.initial_connections.clamp(
                config.min_connections.max(1),
                config.max_connections.max(1),
            ),
            min_marginal_gain: config.min_marginal_gain,
            rtt_inflation_threshold: config.rtt_inflation_threshold,
            control_interval: Duration::from_secs(1),
            last_evaluation: Instant::now(),
            last_bandwidth: 0.0,
            utility_controller: UtilityController::default(),
            last_utility: 0.0,
            exploration_phase: true,
        }
    }

    /// Evaluates current metrics and decides whether to scale worker count.
    pub fn evaluate(
        &mut self,
        model: &NetworkModel,
        is_disk_pressured: bool,
    ) -> ConcurrencyDecision {
        let elapsed = self.last_evaluation.elapsed();
        if elapsed < self.control_interval {
            return ConcurrencyDecision::Hold;
        }
        self.last_evaluation = Instant::now();

        self.utility_controller.update_peak_throughput(model.bandwidth_estimate);
        let current_utility = self.utility_controller.compute_utility(model, is_disk_pressured);

        debug!(
            "Concurrency eval: current_workers={}, bw={:.2}MB/s, rtt_inflation={:.2}%, retries={:.2}%, util={:.2}",
            self.current_target_connections,
            model.bandwidth_estimate / (1024.0 * 1024.0),
            model.rtt_inflation * 100.0,
            model.retry_rate * 100.0,
            current_utility
        );

        // Rule 1: Backpressure or excessive RTT inflation -> scale down immediately
        if is_disk_pressured || model.rtt_inflation > self.rtt_inflation_threshold || model.retry_rate > 0.05 {
            if self.current_target_connections > self.min_connections {
                self.current_target_connections -= 1;
                self.exploration_phase = false;
                info!(
                    "Adaptive Concurrency: Decreasing worker count -> {} (due to RTT inflation/backpressure)",
                    self.current_target_connections
                );
                return ConcurrencyDecision::Decrease;
            }
            return ConcurrencyDecision::Hold;
        }

        // Rule 2: Exploration phase - increase until marginal gain diminishes
        if self.current_target_connections < self.max_connections {
            if self.last_bandwidth > 1024.0 {
                let marginal_gain = (model.bandwidth_estimate - self.last_bandwidth) / self.last_bandwidth;
                if marginal_gain < self.min_marginal_gain && !self.exploration_phase {
                    debug!(
                        "Marginal gain ({:.2}%) < threshold ({:.2}%). Holding concurrency.",
                        marginal_gain * 100.0,
                        self.min_marginal_gain * 100.0
                    );
                    return ConcurrencyDecision::Hold;
                }
            }

            self.last_bandwidth = model.bandwidth_estimate;
            self.last_utility = current_utility;
            self.current_target_connections += 1;
            info!(
                "Adaptive Concurrency: Increasing worker count -> {}",
                self.current_target_connections
            );
            return ConcurrencyDecision::Increase;
        }

        ConcurrencyDecision::Hold
    }

    pub fn current_target_connections(&self) -> usize {
        self.current_target_connections
    }
}
