// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use aurora_core::events::{create_event_bus, DownloadEvent, EventReceiver};
use aurora_core::types::SegmentState;
use state::{AppState, SharedAppState, TelemetryPayload};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.2} GB/s", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.2} MB/s", bytes_per_sec / (1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.2} KB/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}

async fn run_event_and_telemetry_loop(
    mut event_rx: EventReceiver,
    shared_state: SharedAppState,
    app_handle: tauri::AppHandle,
) {
    let mut ticker = tokio::time::interval(Duration::from_millis(200));

    loop {
        tokio::select! {
            // Process incoming download engine events
            Ok(event) = event_rx.recv() => {
                let mut lock = shared_state.lock().await;
                match event {
                    DownloadEvent::MetadataLoaded {
                        download_id,
                        content_length,
                        accepts_ranges,
                        etag,
                        suggested_filename,
                        rtt_ms,
                        server_name,
                        health_rating,
                    } => {
                        if let Some(item) = lock.downloads.iter_mut().find(|d| d.id == download_id) {
                            item.total_bytes = content_length;
                            item.accepts_ranges = accepts_ranges;
                            item.etag = etag;
                            if rtt_ms.is_some() {
                                item.server_rtt_ms = rtt_ms;
                            }
                            if server_name.is_some() {
                                item.server_name = server_name;
                            }
                            if health_rating.is_some() {
                                item.health_rating = health_rating;
                            }
                            if item.filename == "download.bin" || item.filename.is_empty() {
                                item.filename = suggested_filename;
                            }
                        }
                    }
                    DownloadEvent::ThroughputUpdated {
                        download_id,
                        smoothed_rate,
                        eta,
                        downloaded_bytes,
                        total_bytes,
                        ..
                    } => {
                        if let Some(item) = lock.downloads.iter_mut().find(|d| d.id == download_id) {
                            item.current_speed = smoothed_rate;
                            if smoothed_rate > item.peak_speed {
                                item.peak_speed = smoothed_rate;
                            }
                            if item.min_speed == 0.0 || (smoothed_rate > 0.0 && smoothed_rate < item.min_speed) {
                                item.min_speed = smoothed_rate;
                            }
                            item.eta = eta;
                            item.downloaded_bytes = downloaded_bytes;
                            if total_bytes.is_some() {
                                item.total_bytes = total_bytes;
                            }
                        }
                    }
                    DownloadEvent::SegmentStarted {
                        download_id,
                        segment_id,
                        range,
                        worker_id,
                    } => {
                        if let Some(item) = lock.downloads.iter_mut().find(|d| d.id == download_id) {
                            if let Some(seg) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                                seg.state = SegmentState::Downloading;
                                seg.worker_id = Some(worker_id);
                            } else {
                                let mut new_seg = aurora_core::types::Segment::new(segment_id, range);
                                new_seg.state = SegmentState::Downloading;
                                new_seg.worker_id = Some(worker_id);
                                item.segments.push(new_seg);
                            }
                        }
                    }
                    DownloadEvent::SegmentProgress {
                        download_id,
                        segment_id,
                        bytes_downloaded,
                        ..
                    } => {
                        if let Some(item) = lock.downloads.iter_mut().find(|d| d.id == download_id) {
                            if let Some(seg) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                                seg.downloaded = bytes_downloaded;
                                seg.state = SegmentState::Downloading;
                            }
                        }
                    }
                    DownloadEvent::SegmentCompleted {
                        download_id,
                        segment_id,
                        ..
                    } => {
                        if let Some(item) = lock.downloads.iter_mut().find(|d| d.id == download_id) {
                            if let Some(seg) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                                seg.state = SegmentState::Completed;
                            }
                        }
                    }
                    DownloadEvent::Completed { download_id, total_bytes, .. } => {
                        if let Some(item) = lock.downloads.iter_mut().find(|d| d.id == download_id) {
                            item.status = "completed".to_string();
                            item.is_active = false;
                            item.current_speed = 0.0;
                            item.eta = None;
                            item.total_bytes = Some(total_bytes);
                            item.downloaded_bytes = total_bytes;
                            let elapsed = item.start_instant.elapsed();
                            item.elapsed_duration = Some(elapsed);
                            item.end_time_str = Some(chrono::Local::now().format("%H:%M:%S").to_string());
                            for seg in item.segments.iter_mut() {
                                seg.state = SegmentState::Completed;
                            }
                        }
                    }
                    DownloadEvent::Failed { download_id, error } => {
                        if let Some(item) = lock.downloads.iter_mut().find(|d| d.id == download_id) {
                            item.status = format!("failed: {}", error);
                            item.is_active = false;
                            item.current_speed = 0.0;
                        }
                    }
                    _ => {}
                }
            }

            // Periodic telemetry push to React UI
            _ = ticker.tick() => {
                let mut lock = shared_state.lock().await;
                let mut total_speed = 0.0;
                let mut active_count = 0;

                for item in lock.downloads.iter() {
                    if item.is_active && item.status == "downloading" {
                        total_speed += item.current_speed;
                        active_count += 1;
                    }
                }

                if lock.last_speed_sample.elapsed() >= Duration::from_secs(1) {
                    lock.speed_history.remove(0);
                    lock.speed_history.push(total_speed);
                    lock.last_speed_sample = Instant::now();
                }

                let payload = TelemetryPayload {
                    total_speed_bps: total_speed,
                    total_speed_str: format_speed(total_speed),
                    active_tasks: active_count,
                    speed_history: lock.speed_history.clone(),
                    downloads: lock.downloads.iter().map(|d| d.to_dto()).collect(),
                };

                let _ = app_handle.emit("telemetry-update", payload);
            }
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("aurora=info,info")),
        )
        .init();

    let (event_tx, event_rx) = create_event_bus(4096);
    let shared_state: SharedAppState = Arc::new(Mutex::new(AppState::new(event_tx)));

    let state_for_setup = shared_state.clone();

    tauri::Builder::default()
        .manage(shared_state)
        .setup(move |app| {
            let handle = app.handle().clone();

            // Set window icon from bundled default icon
            if let Some(icon) = app.default_window_icon() {
                for (_, window) in app.webview_windows() {
                    let _ = window.set_icon(icon.clone());
                }
            }

            tauri::async_runtime::spawn(async move {
                run_event_and_telemetry_loop(event_rx, state_for_setup, handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_downloads,
            commands::add_download,
            commands::add_batch_downloads,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::restart_download,
            commands::remove_download,
            commands::clear_completed,
            commands::probe_url,
            commands::verify_file_checksum,
            commands::open_download_folder,
            commands::open_file,
            commands::get_config,
            commands::update_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AURORA Kaushal IDM desktop application");
}
