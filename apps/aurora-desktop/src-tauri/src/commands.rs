use crate::state::{
    DownloadItemDto, EngineConfigDto, InternalDownloadItem, ServerCapabilitiesDto, SharedAppState,
};
use aurora_core::config::SchedulerType;
use aurora_core::types::DownloadId;
use aurora_http::{create_http_client, HttpClientConfig, probe_url as http_probe_url};
use aurora_scheduler::engine::DownloadEngine;
use blake3::Hasher as Blake3Hasher;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use url::Url;

fn extract_filename_from_url(url: &Url) -> String {
    let raw = url
        .path_segments()
        .and_then(|segments| segments.filter(|s| !s.is_empty()).last())
        .unwrap_or("download.bin");

    let clean = raw.split('?').next().unwrap_or(raw);
    let clean = clean.split('#').next().unwrap_or(clean);

    if clean.is_empty() {
        "download.bin".to_string()
    } else {
        clean.to_string()
    }
}

fn open_in_system(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", path_str])
            .spawn()
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_downloads(state: tauri::State<'_, SharedAppState>) -> Result<Vec<DownloadItemDto>, String> {
    let lock = state.lock().await;
    Ok(lock.downloads.iter().map(|d| d.to_dto()).collect())
}

#[tauri::command]
pub async fn add_download(
    state: tauri::State<'_, SharedAppState>,
    url: String,
    filename: Option<String>,
    connections: Option<usize>,
    scheduler: Option<String>,
    _checksum: Option<String>,
    _algo: Option<String>,
) -> Result<String, String> {
    let parsed_url = Url::parse(&url).map_err(|e| format!("Invalid URL: {}", e))?;
    let mut lock = state.lock().await;
    let probed = lock.probed_caps.get(&url);

    let final_filename = if let Some(ref cf) = filename {
        if !cf.trim().is_empty() {
            cf.trim().to_string()
        } else if let Some(suggested) = probed.and_then(|p| p.suggested_filename.as_ref()) {
            suggested.clone()
        } else {
            extract_filename_from_url(&parsed_url)
        }
    } else if let Some(suggested) = probed.and_then(|p| p.suggested_filename.as_ref()) {
        suggested.clone()
    } else {
        extract_filename_from_url(&parsed_url)
    };

    let dest_path = lock.config.download_dir.join(&final_filename);
    let download_id = DownloadId::new();
    let pause_signal = Arc::new(AtomicBool::new(false));
    let cancel_signal = Arc::new(AtomicBool::new(false));
    let conns = connections.unwrap_or(lock.config.initial_connections);
    let start_time_str = chrono::Local::now().format("%H:%M:%S").to_string();

    let item = InternalDownloadItem {
        id: download_id,
        url: url.clone(),
        filename: final_filename,
        destination_path: dest_path.to_string_lossy().to_string(),
        total_bytes: probed.and_then(|p| p.content_length),
        downloaded_bytes: 0,
        current_speed: 0.0,
        peak_speed: 0.0,
        min_speed: 0.0,
        eta: None,
        status: "downloading".to_string(),
        active_connections: conns,
        http_version: probed
            .map(|p| format!("{:?}", p.http_version))
            .unwrap_or_else(|| "HTTP/2".to_string()),
        etag: probed.and_then(|p| p.etag.clone()),
        accepts_ranges: probed.map(|p| p.accepts_ranges).unwrap_or(true),
        is_active: true,
        segments: Vec::new(),
        pause_signal: pause_signal.clone(),
        cancel_signal: cancel_signal.clone(),
        start_time_str,
        start_instant: Instant::now(),
        end_time_str: None,
        elapsed_duration: None,
        server_rtt_ms: probed.and_then(|p| p.rtt_ms),
        server_name: probed.and_then(|p| p.server_name.clone()),
        health_rating: probed.and_then(|p| p.health_rating.clone()),
        checksum_result: None,
    };

    lock.downloads.push(item);

    let mut engine_config = lock.config.clone();
    engine_config.initial_connections = conns;
    engine_config.max_connections = conns.max(engine_config.max_connections);
    if let Some(ref sched) = scheduler {
        match sched.as_str() {
            "largest-segment" => engine_config.scheduler_type = SchedulerType::LargestSegment,
            "fixed" => engine_config.scheduler_type = SchedulerType::Fixed,
            "single" => engine_config.scheduler_type = SchedulerType::SingleStream,
            _ => engine_config.scheduler_type = SchedulerType::AuroraEct,
        }
    }
    let tx = lock.event_tx.clone();

    tauri::async_runtime::spawn(async move {
        let engine = DownloadEngine::new(parsed_url, dest_path, engine_config, tx)
            .with_download_id(download_id)
            .with_signals(pause_signal, cancel_signal);
        let _ = engine.run().await;
    });

    Ok(download_id.to_string())
}

#[tauri::command]
pub async fn add_batch_downloads(
    state: tauri::State<'_, SharedAppState>,
    urls: Vec<String>,
    connections: Option<usize>,
) -> Result<Vec<String>, String> {
    let mut added_ids = Vec::new();
    for url in urls {
        if let Ok(id) = add_download(
            state.clone(),
            url,
            None,
            connections,
            None,
            None,
            None,
        ).await {
            added_ids.push(id);
        }
    }
    Ok(added_ids)
}

#[tauri::command]
pub async fn pause_download(state: tauri::State<'_, SharedAppState>, id: String) -> Result<(), String> {
    let mut lock = state.lock().await;
    if let Some(item) = lock.downloads.iter_mut().find(|d| d.id.to_string() == id) {
        item.pause_signal.store(true, Ordering::Relaxed);
        item.status = "paused".to_string();
        item.is_active = false;
        item.current_speed = 0.0;
        return Ok(());
    }
    Err("Download not found".to_string())
}

#[tauri::command]
pub async fn resume_download(state: tauri::State<'_, SharedAppState>, id: String) -> Result<(), String> {
    let mut lock = state.lock().await;
    if let Some(item) = lock.downloads.iter_mut().find(|d| d.id.to_string() == id) {
        if item.status == "paused" {
            item.pause_signal.store(false, Ordering::Relaxed);
            item.status = "downloading".to_string();
            item.is_active = true;

            let parsed_url = Url::parse(&item.url).map_err(|e| e.to_string())?;
            let dest_path = PathBuf::from(&item.destination_path);
            let download_id = item.id;
            let pause_signal = item.pause_signal.clone();
            let cancel_signal = item.cancel_signal.clone();
            let engine_config = lock.config.clone();
            let tx = lock.event_tx.clone();

            tauri::async_runtime::spawn(async move {
                let engine = DownloadEngine::new(parsed_url, dest_path, engine_config, tx)
                    .with_download_id(download_id)
                    .with_signals(pause_signal, cancel_signal);
                let _ = engine.run().await;
            });
            return Ok(());
        }
    }
    Err("Download not found or not paused".to_string())
}

#[tauri::command]
pub async fn cancel_download(state: tauri::State<'_, SharedAppState>, id: String) -> Result<(), String> {
    let mut lock = state.lock().await;
    if let Some(item) = lock.downloads.iter_mut().find(|d| d.id.to_string() == id) {
        item.cancel_signal.store(true, Ordering::Relaxed);
        item.status = "cancelled".to_string();
        item.is_active = false;
        item.current_speed = 0.0;
        return Ok(());
    }
    Err("Download not found".to_string())
}

#[tauri::command]
pub async fn restart_download(state: tauri::State<'_, SharedAppState>, id: String) -> Result<(), String> {
    let (url, filename, conns) = {
        let lock = state.lock().await;
        let item = lock
            .downloads
            .iter()
            .find(|d| d.id.to_string() == id)
            .ok_or_else(|| "Download not found".to_string())?;
        (item.url.clone(), item.filename.clone(), item.active_connections)
    };

    let _ = cancel_download(state.clone(), id.clone()).await;
    let _ = remove_download(state.clone(), id, false).await;
    let _ = add_download(state, url, Some(filename), Some(conns), None, None, None).await?;
    Ok(())
}

#[tauri::command]
pub async fn remove_download(
    state: tauri::State<'_, SharedAppState>,
    id: String,
    delete_file: bool,
) -> Result<(), String> {
    let mut lock = state.lock().await;
    if let Some(pos) = lock.downloads.iter().position(|d| d.id.to_string() == id) {
        let item = lock.downloads.remove(pos);
        item.cancel_signal.store(true, Ordering::Relaxed);
        if delete_file {
            let _ = fs::remove_file(&item.destination_path);
            let part_path = format!("{}.aurora_part", item.destination_path);
            let _ = fs::remove_file(part_path);
        }
        return Ok(());
    }
    Err("Download not found".to_string())
}

#[tauri::command]
pub async fn clear_completed(state: tauri::State<'_, SharedAppState>) -> Result<usize, String> {
    let mut lock = state.lock().await;
    let before = lock.downloads.len();
    lock.downloads.retain(|d| d.status != "completed");
    Ok(before - lock.downloads.len())
}

#[tauri::command]
pub async fn probe_url(
    state: tauri::State<'_, SharedAppState>,
    url: String,
) -> Result<ServerCapabilitiesDto, String> {
    let parsed_url = Url::parse(&url).map_err(|e| format!("Invalid URL: {}", e))?;
    let client = create_http_client(&HttpClientConfig::default())
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let caps = http_probe_url(&client, &parsed_url)
        .await
        .map_err(|e| format!("Probe failed: {}", e))?;

    let dto = ServerCapabilitiesDto {
        url: url.clone(),
        content_length: caps.content_length,
        accepts_ranges: caps.accepts_ranges,
        etag: caps.etag.clone(),
        content_type: caps.mime_type.clone(),
        server_name: caps.server_name.clone(),
        http_version: caps.http_version.clone(),
        rtt_ms: caps.rtt_ms,
        health_rating: caps.health_rating.clone(),
        suggested_filename: caps
            .suggested_filename
            .clone()
            .or_else(|| Some(extract_filename_from_url(&caps.final_url))),
    };

    let mut lock = state.lock().await;
    lock.probed_caps.insert(url, caps);

    Ok(dto)
}

#[tauri::command]
pub async fn verify_file_checksum(
    state: tauri::State<'_, SharedAppState>,
    id: String,
    expected_hash: String,
    algo: String,
) -> Result<String, String> {
    let path_str = {
        let lock = state.lock().await;
        let item = lock
            .downloads
            .iter()
            .find(|d| d.id.to_string() == id)
            .ok_or_else(|| "Download not found".to_string())?;
        item.destination_path.clone()
    };

    let file_bytes = fs::read(&path_str).map_err(|e| format!("Failed to read file: {}", e))?;

    let computed_hash = if algo.to_lowercase() == "blake3" {
        let mut hasher = Blake3Hasher::new();
        hasher.update(&file_bytes);
        hasher.finalize().to_hex().to_string()
    } else {
        let mut hasher = Sha256::new();
        hasher.update(&file_bytes);
        format!("{:x}", hasher.finalize())
    };

    let is_match = computed_hash.eq_ignore_ascii_case(expected_hash.trim());
    let result_msg = if is_match {
        format!("✅ Integrity Verified! ({})", &computed_hash[..12])
    } else {
        format!("❌ Checksum Mismatch! Expected: {} Computed: {}", expected_hash.trim(), &computed_hash[..12])
    };

    let mut lock = state.lock().await;
    if let Some(item) = lock.downloads.iter_mut().find(|d| d.id.to_string() == id) {
        item.checksum_result = Some(result_msg.clone());
    }

    Ok(result_msg)
}

#[tauri::command]
pub async fn open_download_folder(state: tauri::State<'_, SharedAppState>) -> Result<(), String> {
    let dir = {
        let lock = state.lock().await;
        lock.config.download_dir.clone()
    };
    let _ = fs::create_dir_all(&dir);
    open_in_system(&dir.to_string_lossy())
}

#[tauri::command]
pub async fn open_file(filepath: String) -> Result<(), String> {
    open_in_system(&filepath)
}

#[tauri::command]
pub async fn get_config(state: tauri::State<'_, SharedAppState>) -> Result<EngineConfigDto, String> {
    let lock = state.lock().await;
    Ok(EngineConfigDto {
        download_dir: lock.config.download_dir.to_string_lossy().to_string(),
        initial_connections: lock.config.initial_connections,
        max_connections: lock.config.max_connections,
        chunk_size_bytes: lock.config.min_segment_size as usize,
        write_buffer_bytes: lock.config.disk_queue_limit_bytes,
        request_timeout_secs: lock.config.target_segment_duration.as_secs(),
        max_retries: lock.config.max_retries as usize,
    })
}

#[tauri::command]
pub async fn update_config(
    state: tauri::State<'_, SharedAppState>,
    config: EngineConfigDto,
) -> Result<(), String> {
    let mut lock = state.lock().await;
    lock.config.download_dir = PathBuf::from(config.download_dir);
    lock.config.initial_connections = config.initial_connections;
    lock.config.max_connections = config.max_connections;
    lock.config.min_segment_size = config.chunk_size_bytes as u64;
    lock.config.disk_queue_limit_bytes = config.write_buffer_bytes;
    lock.config.target_segment_duration = std::time::Duration::from_secs(config.request_timeout_secs);
    lock.config.max_retries = config.max_retries as u32;
    Ok(())
}

#[tauri::command]
pub async fn get_autostart_status() -> Result<bool, String> {
    Ok(crate::autostart::is_autostart_enabled())
}

#[tauri::command]
pub async fn set_autostart_status(enabled: bool) -> Result<bool, String> {
    crate::autostart::set_autostart(enabled)?;
    Ok(crate::autostart::is_autostart_enabled())
}

#[tauri::command]
pub async fn pause_all_downloads(state: tauri::State<'_, SharedAppState>) -> Result<(), String> {
    let mut lock = state.lock().await;
    for item in lock.downloads.iter_mut() {
        if item.is_active && item.status == "downloading" {
            item.pause_signal.store(true, Ordering::SeqCst);
            item.status = "paused".to_string();
            item.is_active = false;
            item.current_speed = 0.0;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_all_downloads(state: tauri::State<'_, SharedAppState>) -> Result<(), String> {
    let to_resume: Vec<String> = {
        let lock = state.lock().await;
        lock.downloads
            .iter()
            .filter(|d| d.status == "paused" && !d.is_active)
            .map(|d| d.id.to_string())
            .collect()
    };

    for id in to_resume {
        let _ = resume_download(state.clone(), id).await;
    }

    Ok(())
}

#[tauri::command]
pub async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_main_window(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}

