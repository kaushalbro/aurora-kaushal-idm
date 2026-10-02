use aurora_benchmark::mock_server::{MockHttpServer, MockServerConfig};
use aurora_benchmark::runner::BenchmarkRunner;
use std::time::Duration;
use url::Url;

#[tokio::test]
async fn test_live_benchmark_execution() {
    let file_size = 10 * 1024 * 1024; // 10 MB
    let config = MockServerConfig {
        file_size_bytes: file_size,
        simulated_latency: Duration::from_millis(2),
        rate_limit_bytes_per_sec: None,
        inject_429_count: 0,
        inject_500_count: 0,
        change_etag_midway: false,
        reject_ranges: false,
        etag: "\"live-bench-v1\"".to_string(),
    };

    let (_server, server_url) = MockHttpServer::start(config).await;
    let url = Url::parse(&server_url).expect("parse URL");
    let out_dir = std::env::temp_dir().join("aurora_live_bench");
    let _ = std::fs::create_dir_all(&out_dir);

    let runner = BenchmarkRunner::new(url, out_dir, 2);
    let report = runner.run_benchmarks().await.expect("benchmark run");

    println!("\n========================================================");
    println!(" LIVE BENCHMARK RESULTS (Mock HTTP Server, 10MB File)");
    println!("========================================================");
    for s in &report.summary {
        println!(
            "Scheduler: {:<20} | Median: {:>6.3}s | Mean: {:>6.3}s | Speed: {:>7.2} Mbps",
            s.scheduler, s.median_completion_seconds, s.mean_completion_seconds, s.median_average_mbps
        );
    }
    println!("========================================================\n");

    assert_eq!(report.summary.len(), 4);
}
