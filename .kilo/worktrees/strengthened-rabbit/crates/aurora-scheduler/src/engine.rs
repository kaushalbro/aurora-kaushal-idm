use crate::aurora_ect::AuroraEctScheduler;
use crate::concurrency::AdaptiveConcurrencyController;
use crate::fixed::FixedSegmentScheduler;
use crate::largest_segment::LargestSegmentScheduler;
use crate::single_stream::SingleStreamScheduler;
use aurora_core::config::{EngineConfig, SchedulerType};
use aurora_core::error::{DownloadError, Result};
use aurora_core::events::{DownloadEvent, EventSender};
use aurora_core::traits::{RateEstimator, Scheduler, SchedulerAction, StorageEngine, Transport};
use aurora_core::types::{
    ByteRange, DownloadId, DownloadSnapshot, DownloadStatus, Segment, SegmentId, SegmentState,
    WorkerId,
};
use aurora_http::client::HttpClientConfig;
use aurora_http::transport::HttpTransport;
use aurora_metrics::ewma::EwmaRateEstimator;
use aurora_metrics::rtt::RttTracker;
use aurora_recovery::journal::{remove_journal, save_journal};
use aurora_recovery::state::ResumeState;
use aurora_recovery::validator::{validate_resume, ResumeValidationResult};
use aurora_storage::integrity::{verify_file_checksum, HashAlgorithm};
use aurora_storage::writer::PositionedWriter;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info};
use url::Url;

/// Central coordinator for executing and orchestrating an active download.
pub struct DownloadEngine {
    download_id: DownloadId,
    url: Url,
    destination_path: PathBuf,
    config: EngineConfig,
    event_sender: EventSender,
    expected_checksum: Option<(String, HashAlgorithm)>,
    pause_signal: Arc<AtomicBool>,
    cancel_signal: Arc<AtomicBool>,
}

impl DownloadEngine {
    pub fn new(
        url: Url,
        destination_path: PathBuf,
        config: EngineConfig,
        event_sender: EventSender,
    ) -> Self {
        Self {
            download_id: DownloadId::new(),
            url,
            destination_path,
            config,
            event_sender,
            expected_checksum: None,
            pause_signal: Arc::new(AtomicBool::new(false)),
            cancel_signal: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_download_id(mut self, id: DownloadId) -> Self {
        self.download_id = id;
        self
    }

    pub fn with_signals(
        mut self,
        pause_signal: Arc<AtomicBool>,
        cancel_signal: Arc<AtomicBool>,
    ) -> Self {
        self.pause_signal = pause_signal;
        self.cancel_signal = cancel_signal;
        self
    }

    pub fn with_checksum(mut self, hash_hex: String, algorithm: HashAlgorithm) -> Self {
        self.expected_checksum = Some((hash_hex, algorithm));
        self
    }

    pub fn download_id(&self) -> DownloadId {
        self.download_id
    }

    pub fn pause_signal(&self) -> Arc<AtomicBool> {
        self.pause_signal.clone()
    }

    pub fn cancel_signal(&self) -> Arc<AtomicBool> {
        self.cancel_signal.clone()
    }

    pub fn pause(&self) {
        self.pause_signal.store(true, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.cancel_signal.store(true, Ordering::SeqCst);
    }

    /// Executes the complete download pipeline.
    pub async fn run(&self) -> Result<()> {
        let start_time = Instant::now();
        info!(
            "Starting AURORA Download Engine for {} -> {:?}",
            self.url, self.destination_path
        );

        let _ = self.event_sender.send(DownloadEvent::Queued {
            download_id: self.download_id,
        });

        // Step 1: Probe server capabilities
        let _ = self.event_sender.send(DownloadEvent::ProbingStarted {
            download_id: self.download_id,
        });

        let http_config = HttpClientConfig::default();
        let transport = HttpTransport::new(&http_config)?;
        let caps = transport.probe(&self.url).await?;

        let suggested_filename = self
            .destination_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("download.bin")
            .to_string();

        let _ = self.event_sender.send(DownloadEvent::MetadataLoaded {
            download_id: self.download_id,
            content_length: caps.content_length,
            accepts_ranges: caps.accepts_ranges,
            etag: caps.etag.clone(),
            suggested_filename: suggested_filename.clone(),
            rtt_ms: caps.rtt_ms,
            server_name: caps.server_name.clone(),
            health_rating: caps.health_rating.clone(),
        });

        // Step 2: Initialize or resume storage and segments
        let mut segments: Vec<Segment> = Vec::new();
        let mut initial_downloaded = 0u64;

        let journal_path = aurora_recovery::journal::journal_path_for_file(&self.destination_path);
        if journal_path.exists() {
            if let Ok(saved_state) = aurora_recovery::journal::load_journal(&journal_path) {
                if let Ok(ResumeValidationResult::Valid {
                    segments: restored,
                    restored_bytes,
                }) = validate_resume(&saved_state, &caps)
                {
                    segments = restored;
                    initial_downloaded = restored_bytes;
                    info!(
                        "Resuming download from checkpoint: {} bytes already downloaded",
                        initial_downloaded
                    );
                }
            }
        }

        let total_size = caps.content_length;

        // If no valid resume segments, partition fresh segments
        if segments.is_empty() {
            if let Some(size) = total_size {
                if caps.accepts_ranges && self.config.scheduler_type != SchedulerType::SingleStream {
                    let num_initial_chunks = self.config.initial_connections.max(1);
                    let ranges = FixedSegmentScheduler::partition_fixed_ranges(size, num_initial_chunks);
                    for (idx, r) in ranges.into_iter().enumerate() {
                        segments.push(Segment::new((idx + 1) as u32, r));
                    }
                } else {
                    // Single stream fallback
                    segments.push(Segment::new(1, ByteRange::new(0, size.saturating_sub(1))));
                }
            }
        }

        // Initialize PositionedWriter
        let mut writer = PositionedWriter::create(
            self.destination_path.clone(),
            self.config.disk_queue_limit_bytes,
        )
        .await?;

        if let Some(size) = total_size {
            writer.preallocate(size).await?;
        }

        // Step 3: Instantiate Selected Scheduler
        let mut scheduler: Box<dyn Scheduler> = match self.config.scheduler_type {
            SchedulerType::SingleStream => Box::new(SingleStreamScheduler::new()),
            SchedulerType::Fixed => {
                Box::new(FixedSegmentScheduler::new(self.config.initial_connections))
            }
            SchedulerType::LargestSegment => {
                Box::new(LargestSegmentScheduler::new(self.config.min_segment_size))
            }
            SchedulerType::AuroraEct => Box::new(AuroraEctScheduler::new(&self.config)),
        };

        // Channel for worker progress updates: (WorkerId, SegmentId, ChunkBytes, NetworkDuration)
        let (worker_tx, mut worker_rx) = mpsc::channel::<(WorkerId, SegmentId, u64, Duration)>(2048);
        let shared_segments = Arc::new(Mutex::new(segments));
        let active_workers = Arc::new(Mutex::new(HashMap::<WorkerId, tokio::task::JoinHandle<()>>::new()));
        let mut next_segment_id = {
            let segs = shared_segments.lock().await;
            segs.len() as u32 + 1
        };

        let mut rate_estimator = EwmaRateEstimator::new(1.0);
        let mut rtt_tracker = RttTracker::new();
        let mut _concurrency_controller = AdaptiveConcurrencyController::new(&self.config);
        let mut last_checkpoint = Instant::now();
        let mut last_throughput_calc = Instant::now();
        let mut last_dl_bytes = initial_downloaded;

        // Step 4: Core download & scheduling event loop
        let total_downloaded = Arc::new(AtomicU64::new(initial_downloaded));
        let total_downloaded_clone = total_downloaded.clone();

        loop {
            if self.cancel_signal.load(Ordering::Relaxed) {
                info!("Download cancelled by user.");
                let _ = self.event_sender.send(DownloadEvent::StatusChanged {
                    download_id: self.download_id,
                    new_status: DownloadStatus::Cancelled,
                });
                return Err(DownloadError::Cancelled.into());
            }

            if self.pause_signal.load(Ordering::Relaxed) {
                info!("Download paused by user. Persisting checkpoint.");
                let segs = shared_segments.lock().await;
                let resume_state = ResumeState::new(
                    self.download_id,
                    self.url.clone(),
                    caps.final_url.clone(),
                    self.destination_path.clone(),
                    total_size,
                    caps.etag.clone(),
                    caps.last_modified.clone(),
                    &segs,
                );
                let _ = save_journal(&resume_state);
                let _ = self.event_sender.send(DownloadEvent::StatusChanged {
                    download_id: self.download_id,
                    new_status: DownloadStatus::Paused,
                });
                return Ok(());
            }

            // Check if all segments are completed or total bytes fully downloaded
            let all_completed = {
                let segs = shared_segments.lock().await;
                let total_bytes_done = total_downloaded.load(Ordering::Relaxed);
                let size_done = if let Some(total) = total_size {
                    total > 0 && total_bytes_done >= total
                } else {
                    false
                };
                (!segs.is_empty() && segs.iter().all(|s| s.is_completed())) || size_done
            };

            if all_completed {
                break;
            }

            // Process worker chunk messages
            while let Ok((_w_id, seg_id, bytes_count, net_dt)) = worker_rx.try_recv() {
                total_downloaded_clone.fetch_add(bytes_count, Ordering::Relaxed);
                if net_dt.as_secs_f64() > 0.0 {
                    rtt_tracker.record_sample(net_dt);
                }

                let mut segs = shared_segments.lock().await;
                if let Some(seg) = segs.iter_mut().find(|s| s.id == seg_id) {
                    seg.downloaded += bytes_count;
                    if seg.downloaded >= seg.range.len() {
                        seg.state = SegmentState::Completed;
                        let _ = self.event_sender.send(DownloadEvent::SegmentCompleted {
                            download_id: self.download_id,
                            segment_id: seg_id,
                            bytes_written: seg.downloaded,
                        });
                    }
                }
            }

            // Update rate estimator based on true wall-clock elapsed time
            let now = Instant::now();
            let dt = now.duration_since(last_throughput_calc);
            if dt >= Duration::from_millis(100) {
                let current_total = total_downloaded.load(Ordering::Relaxed);
                let bytes_delta = current_total.saturating_sub(last_dl_bytes);
                rate_estimator.update(bytes_delta, dt);
                last_dl_bytes = current_total;
                last_throughput_calc = now;
            }

            // Checkpoint periodically
            if last_checkpoint.elapsed() >= self.config.checkpoint_interval {
                last_checkpoint = Instant::now();
                let segs = shared_segments.lock().await;
                let resume_state = ResumeState::new(
                    self.download_id,
                    self.url.clone(),
                    caps.final_url.clone(),
                    self.destination_path.clone(),
                    total_size,
                    caps.etag.clone(),
                    caps.last_modified.clone(),
                    &segs,
                );
                let _ = save_journal(&resume_state);
            }

            // Prepare snapshot for scheduler
            // Evaluate Scheduler actions in a loop to immediately dispatch all available work
            loop {
                let snapshot = {
                    let segs = shared_segments.lock().await;
                    let active_conns = {
                        let workers = active_workers.lock().await;
                        workers.len()
                    };
                    let dl_bytes = total_downloaded.load(Ordering::Relaxed);
                    let current_rate = rate_estimator.smoothed_rate();
                    let eta = if let Some(total) = total_size {
                        if current_rate > 1024.0 {
                            let remaining = total.saturating_sub(dl_bytes);
                            Some(Duration::from_secs_f64(remaining as f64 / current_rate))
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    DownloadSnapshot {
                        id: self.download_id,
                        url: self.url.clone(),
                        filename: suggested_filename.clone(),
                        destination_path: self.destination_path.clone(),
                        total_bytes: total_size,
                        downloaded_bytes: dl_bytes,
                        status: DownloadStatus::Downloading,
                        aggregate_rate: current_rate,
                        eta,
                        active_connections: active_conns,
                        segments: segs.clone(),
                        server_capabilities: Some(caps.clone()),
                    }
                };

                let action = scheduler.next_action(&snapshot);
                match action {
                    SchedulerAction::StartSegment { worker_id, range } => {
                        let seg_id = {
                            let mut segs = shared_segments.lock().await;
                            if let Some(s) = segs.iter_mut().find(|s| s.range == range) {
                                s.state = SegmentState::Downloading;
                                s.worker_id = Some(worker_id);
                                s.id
                            } else {
                                let id = next_segment_id;
                                next_segment_id += 1;
                                let mut new_seg = Segment::new(id, range);
                                new_seg.state = SegmentState::Downloading;
                                new_seg.worker_id = Some(worker_id);
                                segs.push(new_seg);
                                id
                            }
                        };

                        let _ = self.event_sender.send(DownloadEvent::SegmentStarted {
                            download_id: self.download_id,
                            segment_id: seg_id,
                            range,
                            worker_id,
                        });

                        self.spawn_worker_task(
                            worker_id,
                            seg_id,
                            range,
                            transport.clone(),
                            caps.etag.clone(),
                            caps.last_modified.clone(),
                            worker_tx.clone(),
                            shared_segments.clone(),
                            active_workers.clone(),
                            writer.backpressure_controller().clone(),
                            self.destination_path.clone(),
                        )
                        .await;
                    }
                    SchedulerAction::SplitAndAssign {
                        target_segment_id,
                        new_range_for_existing,
                        stolen_range_for_new_worker,
                        idle_worker_id,
                    } => {
                        let new_seg_id = next_segment_id;
                        next_segment_id += 1;

                        {
                            let mut segs = shared_segments.lock().await;
                            if let Some(target) = segs.iter_mut().find(|s| s.id == target_segment_id) {
                                target.range = new_range_for_existing;
                            }
                            let mut stolen_seg = Segment::new(new_seg_id, stolen_range_for_new_worker);
                            stolen_seg.state = SegmentState::Downloading;
                            stolen_seg.worker_id = Some(idle_worker_id);
                            segs.push(stolen_seg);
                        }

                        let _ = self.event_sender.send(DownloadEvent::SegmentSplit {
                            download_id: self.download_id,
                            original_segment_id: target_segment_id,
                            new_segment_id: new_seg_id,
                            left_range: new_range_for_existing,
                            right_range: stolen_range_for_new_worker,
                            reason: "AURORA-ECT rate rebalance".to_string(),
                        });

                        let _ = self.event_sender.send(DownloadEvent::SegmentStarted {
                            download_id: self.download_id,
                            segment_id: new_seg_id,
                            range: stolen_range_for_new_worker,
                            worker_id: idle_worker_id,
                        });

                        self.spawn_worker_task(
                            idle_worker_id,
                            new_seg_id,
                            stolen_range_for_new_worker,
                            transport.clone(),
                            caps.etag.clone(),
                            caps.last_modified.clone(),
                            worker_tx.clone(),
                            shared_segments.clone(),
                            active_workers.clone(),
                            writer.backpressure_controller().clone(),
                            self.destination_path.clone(),
                        )
                        .await;
                    }
                    SchedulerAction::DuplicateTail {
                        source_segment_id: _,
                        hedge_range,
                        worker_id,
                    } => {
                        let hedge_seg_id = next_segment_id;
                        next_segment_id += 1;

                        {
                            let mut segs = shared_segments.lock().await;
                            let mut hedge_seg = Segment::new(hedge_seg_id, hedge_range);
                            hedge_seg.state = SegmentState::Downloading;
                            hedge_seg.worker_id = Some(worker_id);
                            segs.push(hedge_seg);
                        }

                        let _ = self.event_sender.send(DownloadEvent::SegmentStarted {
                            download_id: self.download_id,
                            segment_id: hedge_seg_id,
                            range: hedge_range,
                            worker_id,
                        });

                        self.spawn_worker_task(
                            worker_id,
                            hedge_seg_id,
                            hedge_range,
                            transport.clone(),
                            caps.etag.clone(),
                            caps.last_modified.clone(),
                            worker_tx.clone(),
                            shared_segments.clone(),
                            active_workers.clone(),
                            writer.backpressure_controller().clone(),
                            self.destination_path.clone(),
                        )
                        .await;
                    }
                    _ => break,
                }
            }

            // Emit ThroughputUpdated event every tick
            {
                let dl_bytes = total_downloaded.load(Ordering::Relaxed);
                let current_rate = rate_estimator.smoothed_rate();
                let eta = if let Some(total) = total_size {
                    if current_rate > 1024.0 {
                        let remaining = total.saturating_sub(dl_bytes);
                        Some(Duration::from_secs_f64(remaining as f64 / current_rate))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let _ = self.event_sender.send(DownloadEvent::ThroughputUpdated {
                    download_id: self.download_id,
                    instantaneous_rate: rate_estimator.instantaneous_rate(),
                    smoothed_rate: current_rate,
                    eta,
                    downloaded_bytes: dl_bytes,
                    total_bytes: total_size,
                });
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Abort any remaining background worker tasks (e.g. redundant hedge workers)
        {
            let workers = active_workers.lock().await;
            for handle in workers.values() {
                handle.abort();
            }
        }

        let elapsed = start_time.elapsed();
        let total_bytes_dl = total_downloaded.load(Ordering::Relaxed);
        let avg_speed = if elapsed.as_secs_f64() > 0.0 {
            total_bytes_dl as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        // Step 5: Integrity Verification if expected checksum provided
        let mut actual_checksum = None;
        if let Some((expected_hex, algo)) = &self.expected_checksum {
            info!("Verifying downloaded file checksum against {:?}...", algo);
            verify_file_checksum(&self.destination_path, expected_hex, *algo)?;
            actual_checksum = Some(expected_hex.clone());
        }

        info!(
            "Download completed successfully in {:.2?}s! Average speed: {:.2} MB/s",
            elapsed,
            avg_speed / (1024.0 * 1024.0)
        );

        // Instant completion event dispatch — 0.0s UI latency
        let _ = self.event_sender.send(DownloadEvent::Completed {
            download_id: self.download_id,
            total_bytes: total_bytes_dl,
            elapsed,
            average_speed: avg_speed,
            checksum: actual_checksum,
        });

        // Non-blocking asynchronous storage finalization and recovery journal cleanup
        let dest_path_clone = self.destination_path.clone();
        tokio::spawn(async move {
            let _ = writer.finalize().await;
            let _ = remove_journal(&dest_path_clone);
        });

        Ok(())
    }

    async fn spawn_worker_task(
        &self,
        worker_id: WorkerId,
        segment_id: SegmentId,
        range: ByteRange,
        transport: HttpTransport,
        etag: Option<String>,
        last_modified: Option<String>,
        worker_tx: mpsc::Sender<(WorkerId, SegmentId, u64, Duration)>,
        shared_segments: Arc<Mutex<Vec<Segment>>>,
        active_workers: Arc<Mutex<HashMap<WorkerId, tokio::task::JoinHandle<()>>>>,
        backpressure: aurora_storage::BackpressureController,
        dest_path: PathBuf,
    ) {
        let url = self.url.clone();
        let workers_map = active_workers.clone();
        let handle = tokio::spawn(async move {
            debug!("Worker {} started for range {}", worker_id, range);
            let fetch_res = transport
                .fetch_range(&url, range, etag.as_deref(), last_modified.as_deref())
                .await;

            match fetch_res {
                Ok(stream_handle) => {
                    let mut stream = stream_handle.response.bytes_stream();
                    let mut current_offset = range.start;
                    let mut last_recv_time = Instant::now();

                    // Open direct writer for this thread
                    if let Ok(file) = std::fs::OpenOptions::new().write(true).open(&dest_path) {
                        const BATCH_SIZE: usize = 256 * 1024; // 256 KB memory buffer aggregation
                        let mut write_buf: Vec<u8> = Vec::with_capacity(BATCH_SIZE + 64 * 1024);
                        let mut buffer_start_offset = range.start;
                        let mut unnotified_bytes = 0u64;
                        let mut unnotified_dt = Duration::ZERO;

                        while let Some(chunk_res) = stream.next().await {
                            let now = Instant::now();
                            let net_dt = now.duration_since(last_recv_time);
                            last_recv_time = now;

                            match chunk_res {
                                Ok(bytes) => {
                                    let chunk_len = bytes.len() as u64;
                                    write_buf.extend_from_slice(&bytes);
                                    current_offset += chunk_len;
                                    unnotified_bytes += chunk_len;
                                    unnotified_dt += net_dt;

                                    // Check if segment has been truncated by rebalance
                                    let is_done = {
                                        let segs = shared_segments.lock().await;
                                        if let Some(s) = segs.iter().find(|s| s.id == segment_id) {
                                            current_offset > s.range.end
                                        } else {
                                            true
                                        }
                                    };

                                    // Flush 256 KB batch to disk in a single aligned system call
                                    if write_buf.len() >= BATCH_SIZE || is_done {
                                        backpressure.wait_if_pressured().await;

                                        if let Err(e) =
                                            aurora_storage::platform::write_all_at(&file, buffer_start_offset, &write_buf)
                                        {
                                            error!("Worker {} write error: {:?}", worker_id, e);
                                            break;
                                        }

                                        buffer_start_offset += write_buf.len() as u64;
                                        write_buf.clear();

                                        let _ = worker_tx.send((worker_id, segment_id, unnotified_bytes, unnotified_dt)).await;
                                        unnotified_bytes = 0;
                                        unnotified_dt = Duration::ZERO;
                                    } else if unnotified_bytes >= 64 * 1024 {
                                        // Telemetry milestone
                                        let _ = worker_tx.send((worker_id, segment_id, unnotified_bytes, unnotified_dt)).await;
                                        unnotified_bytes = 0;
                                        unnotified_dt = Duration::ZERO;
                                    }

                                    if is_done {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Worker {} stream error: {:?}", worker_id, e);
                                    break;
                                }
                            }
                        }

                        // Final trailing flush if any remaining bytes in write_buf
                        if !write_buf.is_empty() {
                            let _ = aurora_storage::platform::write_all_at(&file, buffer_start_offset, &write_buf);
                            if unnotified_bytes > 0 {
                                let _ = worker_tx.send((worker_id, segment_id, unnotified_bytes, unnotified_dt)).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Worker {} failed to connect range {}: {:?}", worker_id, range, e);
                }
            }

            let mut workers = workers_map.lock().await;
            workers.remove(&worker_id);
            debug!("Worker {} finished.", worker_id);
        });

        let mut workers = active_workers.lock().await;
        workers.insert(worker_id, handle);
    }
}
