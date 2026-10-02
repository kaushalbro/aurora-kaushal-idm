pub mod mock_server;
pub mod runner;
pub mod simulator;

pub use mock_server::{MockHttpServer, MockServerConfig};
pub use runner::{BenchmarkReport, BenchmarkRunResult, BenchmarkRunner, SchedulerSummary};
pub use simulator::{SchedulerSimulator, SimulationResult};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_mock_http_server_range_requests() {
        let config = MockServerConfig {
            file_size_bytes: 1024 * 1024, // 1 MB
            simulated_latency: Duration::ZERO,
            rate_limit_bytes_per_sec: None,
            inject_429_count: 0,
            inject_500_count: 0,
            change_etag_midway: false,
            reject_ranges: false,
            etag: "\"mock-v1\"".to_string(),
        };

        let (_server, url_str) = MockHttpServer::start(config).await;
        let client = reqwest::Client::new();

        // Send Range 0-99 request
        let resp = client
            .get(&url_str)
            .header(reqwest::header::RANGE, "bytes=0-99")
            .send()
            .await
            .expect("send request");

        assert_eq!(resp.status(), reqwest::StatusCode::PARTIAL_CONTENT);
        let bytes = resp.bytes().await.expect("read bytes");
        assert_eq!(bytes.len(), 100);
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[1], 1);
    }

    #[test]
    fn test_simulator_heterogeneous_straggler_reduction() {
        // Workers: 100 MB/s, 30 MB/s, 10 MB/s (extreme straggler scenario)
        let sim = SchedulerSimulator::new(1024 * 1024 * 1024, &[100.0, 30.0, 10.0]); // 1 GB file
        let results = sim.run_all_simulations();

        let fixed = results.iter().find(|r| r.scheduler_type == "Fixed Segmentation").unwrap();
        let ect = results.iter().find(|r| r.scheduler_type.contains("ECT")).unwrap();

        // AURORA-ECT must finish faster than Fixed segmentation when worker speeds are non-uniform
        assert!(
            ect.completion_time_seconds < fixed.completion_time_seconds,
            "ECT ({:.2}s) should complete faster than Fixed ({:.2}s)",
            ect.completion_time_seconds,
            fixed.completion_time_seconds
        );

        assert!(ect.efficiency_pct > fixed.efficiency_pct);
    }
}
