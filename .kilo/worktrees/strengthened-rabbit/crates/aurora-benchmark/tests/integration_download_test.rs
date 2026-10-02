use aurora_benchmark::mock_server::{MockHttpServer, MockServerConfig};
use aurora_core::config::{EngineConfig, SchedulerType};
use aurora_core::events::create_event_bus;
use aurora_scheduler::engine::DownloadEngine;
use aurora_storage::integrity::{compute_sha256, verify_file_checksum, HashAlgorithm};
use std::time::Duration;
use tempfile::NamedTempFile;
use url::Url;

#[tokio::test]
async fn test_full_download_pipeline_with_mock_server() {
    let file_size = 5 * 1024 * 1024; // 5 MB
    let config = MockServerConfig {
        file_size_bytes: file_size,
        simulated_latency: Duration::from_millis(5),
        rate_limit_bytes_per_sec: None,
        inject_429_count: 0,
        inject_500_count: 0,
        change_etag_midway: false,
        reject_ranges: false,
        etag: "\"aurora-integration-v1\"".to_string(),
    };

    let (_server, server_url) = MockHttpServer::start(config).await;
    let url = Url::parse(&server_url).expect("parse URL");

    let temp_file = NamedTempFile::new().expect("temp file");
    let dest_path = temp_file.path().to_path_buf();

    let mut engine_config = EngineConfig::default();
    engine_config.scheduler_type = SchedulerType::AuroraEct;
    engine_config.initial_connections = 4;
    engine_config.min_segment_size = 64 * 1024; // 64 KB

    let (tx, _rx) = create_event_bus(512);
    let engine = DownloadEngine::new(url, dest_path.clone(), engine_config, tx);

    engine.run().await.expect("download failed");

    // Verify downloaded file size
    let metadata = std::fs::metadata(&dest_path).expect("read metadata");
    assert_eq!(metadata.len(), file_size as u64);

    // Verify byte content
    let content = std::fs::read(&dest_path).expect("read content");
    assert_eq!(content.len(), file_size);
    for (i, &b) in content.iter().enumerate() {
        assert_eq!(b, (i % 251) as u8, "byte mismatch at offset {}", i);
    }

    // Verify checksum
    let hash = compute_sha256(&dest_path).expect("compute sha256");
    assert!(!hash.is_empty());
    let is_valid = verify_file_checksum(&dest_path, &hash, HashAlgorithm::Sha256)
        .expect("verify hash");
    assert!(is_valid);
}

#[tokio::test]
async fn test_all_schedulers_against_mock_server() {
    let file_size = 2 * 1024 * 1024; // 2 MB
    let config = MockServerConfig {
        file_size_bytes: file_size,
        simulated_latency: Duration::ZERO,
        rate_limit_bytes_per_sec: None,
        inject_429_count: 0,
        inject_500_count: 0,
        change_etag_midway: false,
        reject_ranges: false,
        etag: "\"sched-test-v1\"".to_string(),
    };

    let (_server, server_url) = MockHttpServer::start(config).await;
    let url = Url::parse(&server_url).expect("parse URL");

    let schedulers = [
        SchedulerType::SingleStream,
        SchedulerType::Fixed,
        SchedulerType::LargestSegment,
        SchedulerType::AuroraEct,
    ];

    for scheduler in schedulers {
        let temp_file = NamedTempFile::new().expect("temp file");
        let dest_path = temp_file.path().to_path_buf();

        let mut engine_config = EngineConfig::default();
        engine_config.scheduler_type = scheduler;
        engine_config.initial_connections = 4;
        engine_config.min_segment_size = 32 * 1024;

        let (tx, _rx) = create_event_bus(512);
        let engine = DownloadEngine::new(url.clone(), dest_path.clone(), engine_config, tx);

        engine.run().await.expect("download failed for scheduler");

        let metadata = std::fs::metadata(&dest_path).expect("read metadata");
        assert_eq!(metadata.len(), file_size as u64);
    }
}
