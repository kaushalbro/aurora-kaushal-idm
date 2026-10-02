pub mod ewma;
pub mod network_model;
pub mod platform;
pub mod rtt;
pub mod utility;

pub use ewma::EwmaRateEstimator;
pub use network_model::NetworkModel;
pub use platform::{current_process_metrics, ProcessMetrics};
pub use rtt::RttTracker;
pub use utility::{UtilityController, UtilityWeights};

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::traits::RateEstimator;
    use std::time::Duration;

    #[test]
    fn test_ewma_rate_estimation() {
        let mut ewma = EwmaRateEstimator::new(1.0);

        // 10 MB in 1 second = 10 MB/s
        ewma.update(10 * 1024 * 1024, Duration::from_secs(1));
        let rate = ewma.smoothed_rate();
        assert!((rate - 10.0 * 1024.0 * 1024.0).abs() < 1000.0);

        // Next 5 MB in 1 second = 5 MB/s, rate should decay smoothly
        ewma.update(5 * 1024 * 1024, Duration::from_secs(1));
        assert!(ewma.smoothed_rate() < 10.0 * 1024.0 * 1024.0);
        assert!(ewma.smoothed_rate() > 5.0 * 1024.0 * 1024.0);
    }

    #[test]
    fn test_rtt_tracker_inflation() {
        let mut tracker = RttTracker::new();

        tracker.record_sample(Duration::from_millis(40));
        assert_eq!(tracker.min_rtt(), Duration::from_millis(40));
        assert_eq!(tracker.rtt_inflation(), 0.0);

        // Injected high queue latency: 120ms (3x min)
        for _ in 0..15 {
            tracker.record_sample(Duration::from_millis(120));
        }

        assert!(tracker.rtt_inflation() > 1.0); // Inflation > 100%
    }

    #[test]
    fn test_utility_controller_evaluation() {
        let mut controller = UtilityController::default();
        controller.update_peak_throughput(100.0 * 1024.0 * 1024.0); // 100 MB/s

        let mut good_model = NetworkModel::new();
        good_model.bandwidth_estimate = 90.0 * 1024.0 * 1024.0;
        good_model.rtt_inflation = 0.05;
        good_model.retry_rate = 0.0;

        let good_util = controller.compute_utility(&good_model, false);

        let mut bad_model = NetworkModel::new();
        bad_model.bandwidth_estimate = 92.0 * 1024.0 * 1024.0; // slight speed gain
        bad_model.rtt_inflation = 2.5; // huge queue bloat
        bad_model.retry_rate = 0.2; // 20% retries

        let bad_util = controller.compute_utility(&bad_model, true);

        assert!(good_util > bad_util);
    }
}
