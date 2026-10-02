use anyhow::Context;
use aurora_benchmark::mock_server::{MockHttpServer, MockServerConfig};
use aurora_benchmark::runner::BenchmarkRunner;
use aurora_benchmark::simulator::SchedulerSimulator;
use aurora_core::config::{EngineConfig, SchedulerType};
use aurora_core::events::{create_event_bus, DownloadEvent};
use aurora_core::platform::default_download_dir;
use aurora_scheduler::engine::DownloadEngine;
use aurora_storage::integrity::HashAlgorithm;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use url::Url;

#[derive(Parser)]
#[command(name = "aurora")]
#[command(author = "AURORA Development Team")]
#[command(version = "0.2.0")]
#[command(about = "High-Performance Research Download Manager in pure Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download a file from URL with high performance and adaptive scheduling
    Download {
        /// URL of the remote file
        url: String,

        /// Output file path (defaults to filename from URL in downloads folder)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Scheduling algorithm to use
        #[arg(short, long, value_enum, default_value = "aurora-ect")]
        scheduler: CliScheduler,

        /// Number of parallel connections/segments
        #[arg(short, long, default_value = "8")]
        connections: usize,

        /// Target segment duration in seconds
        #[arg(long, default_value = "4")]
        segment_duration: u64,

        /// Expected SHA-256 or BLAKE3 checksum for integrity verification
        #[arg(long)]
        checksum: Option<String>,

        /// Checksum hash algorithm (sha256 or blake3)
        #[arg(long, default_value = "sha256")]
        hash_algo: String,
    },

    /// Run rigorous comparative benchmarks across all scheduling algorithms
    Benchmark {
        /// Target URL or test server endpoint
        url: String,

        /// Output directory for test files
        #[arg(short, long)]
        output_dir: Option<PathBuf>,

        /// Number of runs/iterations per scheduler
        #[arg(short, long, default_value = "3")]
        runs: usize,

        /// Report output format (json or csv)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Save report to file path
        #[arg(long)]
        report_file: Option<PathBuf>,
    },

    /// Run pure mathematical discrete-event simulation of schedulers
    Simulate {
        /// Comma-separated worker speeds in MB/s (e.g., 100,30,10)
        #[arg(short, long, default_value = "100,30,10")]
        workers: String,

        /// Synthetic file size in megabytes (MB)
        #[arg(short, long, default_value = "1024")]
        file_size_mb: u64,
    },

    /// Start local Mock HTTP Test Server for testing and benchmarking
    TestServer {
        /// Synthetic file size in megabytes (MB)
        #[arg(short, long, default_value = "100")]
        size_mb: usize,

        /// Artificial per-stream bandwidth limit in MB/s (optional)
        #[arg(short, long)]
        rate_limit_mb: Option<usize>,

        /// Simulated latency in milliseconds
        #[arg(short, long, default_value = "0")]
        latency_ms: u64,
    },

    /// Build and package the WebAssembly Chrome Extension
    BuildExtension,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum CliScheduler {
    SingleStream,
    Fixed,
    LargestSegment,
    AuroraEct,
}

impl From<CliScheduler> for SchedulerType {
    fn from(c: CliScheduler) -> Self {
        match c {
            CliScheduler::SingleStream => SchedulerType::SingleStream,
            CliScheduler::Fixed => SchedulerType::Fixed,
            CliScheduler::LargestSegment => SchedulerType::LargestSegment,
            CliScheduler::AuroraEct => SchedulerType::AuroraEct,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("aurora=info,info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Download {
            url,
            output,
            scheduler,
            connections,
            segment_duration,
            checksum,
            hash_algo,
        } => {
            let parsed_url = Url::parse(&url).context("Failed to parse download URL")?;
            let dest_path = output.unwrap_or_else(|| {
                let filename = parsed_url
                    .path_segments()
                    .and_then(|mut s| s.next_back())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("download.bin");
                default_download_dir().join(filename)
            });

            let mut config = EngineConfig::default();
            config.scheduler_type = scheduler.into();
            config.initial_connections = connections;
            config.max_connections = connections.max(16);
            config.target_segment_duration = Duration::from_secs(segment_duration);

            println!("========================================================");
            println!(" AURORA Download Manager v0.1.0 (Rust Engine)");
            println!(" URL:         {}", parsed_url);
            println!(" Destination: {:?}", dest_path);
            println!(" Scheduler:   {:?}", config.scheduler_type);
            println!(" Connections: {}", connections);
            println!("========================================================");

            let (tx, mut rx) = create_event_bus(512);

            let mut engine = DownloadEngine::new(parsed_url, dest_path.clone(), config, tx);
            if let Some(expected_hex) = checksum {
                let algo = HashAlgorithm::from_str_name(&hash_algo)
                    .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                engine = engine.with_checksum(expected_hex, algo);
            }

            let pb = ProgressBar::new(100);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );

            let pb_clone = pb.clone();
            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    match event {
                        DownloadEvent::MetadataLoaded { content_length, .. } => {
                            if let Some(total) = content_length {
                                pb_clone.set_length(total);
                            }
                        }
                        DownloadEvent::ThroughputUpdated {
                            downloaded_bytes, ..
                        } => {
                            pb_clone.set_position(downloaded_bytes);
                        }
                        DownloadEvent::Completed { .. } => {
                            pb_clone.finish_with_message("Download Complete!");
                        }
                        _ => {}
                    }
                }
            });

            engine.run().await.context("Download failed")?;
            println!("\nSaved to: {:?}", dest_path);
        }

        Commands::Benchmark {
            url,
            output_dir,
            runs,
            format,
            report_file,
        } => {
            let parsed_url = Url::parse(&url).context("Failed to parse target URL")?;
            let out_dir = output_dir.unwrap_or_else(|| std::env::temp_dir().join("aurora_bench"));
            std::fs::create_dir_all(&out_dir)?;

            println!("========================================================");
            println!(" AURORA Benchmark Suite");
            println!(" Target URL:  {}", parsed_url);
            println!(" Iterations:  {} per scheduler", runs);
            println!(" Output Dir:  {:?}", out_dir);
            println!("========================================================");

            let runner = BenchmarkRunner::new(parsed_url, out_dir, runs);
            let report = runner.run_benchmarks().await.context("Benchmark failed")?;

            println!("\n=== Benchmark Summary ===");
            for s in &report.summary {
                println!(
                    "Scheduler: {:<20} | Median: {:>6.2}s | Mean: {:>6.2}s | StdDev: {:>5.2}s | Speed: {:>7.2} Mbps",
                    s.scheduler, s.median_completion_seconds, s.mean_completion_seconds, s.std_dev_seconds, s.median_average_mbps
                );
            }

            let output_str = if format.eq_ignore_ascii_case("csv") {
                BenchmarkRunner::export_csv(&report)
            } else {
                BenchmarkRunner::export_json(&report)
            };

            if let Some(path) = report_file {
                std::fs::write(&path, output_str)?;
                println!("\nSaved report to: {:?}", path);
            } else {
                println!("\n--- Report Output ---\n{}", output_str);
            }
        }

        Commands::Simulate {
            workers,
            file_size_mb,
        } => {
            let speeds: Vec<f64> = workers
                .split(',')
                .filter_map(|s| s.trim().parse::<f64>().ok())
                .collect();

            if speeds.is_empty() {
                anyhow::bail!("Must provide at least one valid worker speed in MB/s");
            }

            let file_size_bytes = file_size_mb * 1024 * 1024;
            let sim = SchedulerSimulator::new(file_size_bytes, &speeds);
            let results = sim.run_all_simulations();

            println!("==========================================================================");
            println!(" AURORA Discrete-Event Scheduler Simulation");
            println!(" File Size:     {} MB ({} bytes)", file_size_mb, file_size_bytes);
            println!(" Worker Speeds: {:?} MB/s", speeds);
            println!(" Theoretical Optimal Time: {:.2}s", sim.theoretical_optimal());
            println!("==========================================================================");

            for r in results {
                println!(
                    "Scheduler: {:<32} | Time: {:>6.2}s | Efficiency: {:>5.1}% | Splits: {:>2} | Idle: {:>5.2}s",
                    r.scheduler_type, r.completion_time_seconds, r.efficiency_pct, r.total_splits, r.worker_idle_time_seconds
                );
            }
        }

        Commands::TestServer {
            size_mb,
            rate_limit_mb,
            latency_ms,
        } => {
            let config = MockServerConfig {
                file_size_bytes: size_mb * 1024 * 1024,
                simulated_latency: Duration::from_millis(latency_ms),
                rate_limit_bytes_per_sec: rate_limit_mb.map(|mb| mb * 1024 * 1024),
                inject_429_count: 0,
                inject_500_count: 0,
                change_etag_midway: false,
                reject_ranges: false,
                etag: "\"aurora-test-server-v1\"".to_string(),
            };

            let (_server, url) = MockHttpServer::start(config).await;
            println!("========================================================");
            println!(" AURORA Mock HTTP Test Server Started");
            println!(" Endpoint:    {}", url);
            println!(" Size:        {} MB", size_mb);
            if let Some(rate) = rate_limit_mb {
                println!(" Rate Limit:  {} MB/s", rate);
            }
            println!(" Latency:     {} ms", latency_ms);
            println!(" Press Ctrl+C to stop.");
            println!("========================================================");

            tokio::signal::ctrl_c().await?;
            println!("\nServer stopped.");
        }

        Commands::BuildExtension => {
            let status = std::process::Command::new("bash")
                .arg("scripts/build_extension.sh")
                .status()
                .context("Failed to execute extension build script")?;
            if !status.success() {
                anyhow::bail!("Extension build failed with exit code: {:?}", status.code());
            }
        }
    }

    Ok(())
}
