use crate::sorting::{filter_and_sort_indices, CategoryFilter, SortColumn, SortDirection};
use crate::theme::{self, icon_button, pill_badge, primary_button, secondary_button};
use crate::views::batch_view::{render_batch_modal, BatchModalState};
use crate::views::detail_view::render_detail_modal;
use crate::views::graph_view::render_speed_graph;
use crate::views::queue_view::render_queue_table;
use crate::views::segment_view::render_segment_visualization;
use crate::views::settings_view::render_settings_modal;
use crate::views::sidebar_view::render_sidebar;
use aurora_core::config::EngineConfig;
use aurora_core::events::{create_event_bus, DownloadEvent, EventReceiver, EventSender};
use aurora_core::types::{DownloadId, Segment, SegmentState};
use aurora_scheduler::engine::DownloadEngine;
use egui::{Color32, Context, Margin, RichText, Rounding, SidePanel, Stroke, TopBottomPanel};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use url::Url;

pub struct GuiDownloadItem {
    pub id: DownloadId,
    pub url: String,
    pub filename: String,
    pub destination_path: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub current_speed: f64,
    pub peak_speed: f64,
    pub min_speed: f64,
    pub eta: Option<Duration>,
    pub status: String,
    pub active_connections: usize,
    pub http_version: String,
    pub etag: Option<String>,
    pub accepts_ranges: bool,
    pub is_active: bool,
    pub segments: Vec<Segment>,
    pub pause_signal: Arc<AtomicBool>,
    pub cancel_signal: Arc<AtomicBool>,

    // Action requests from buttons & context menu
    pub requested_pause: bool,
    pub requested_resume: bool,
    pub requested_restart: bool,
    pub requested_open_file: bool,
    pub requested_open_folder: bool,
    pub requested_remove: bool,
    pub requested_delete_file: bool,
    pub requested_reprobe: bool,
    pub requested_inspect: bool,

    pub start_time_str: String,
    pub start_instant: std::time::Instant,
    pub end_time_str: Option<String>,
    pub elapsed_duration: Option<Duration>,
    pub server_rtt_ms: Option<u64>,
    pub server_name: Option<String>,
    pub health_rating: Option<String>,

    // Cryptographic Checksum fields
    pub checksum_input: String,
    pub checksum_result: Option<String>,
}

pub struct AuroraApp {
    pub config: EngineConfig,
    pub downloads: Vec<GuiDownloadItem>,
    pub selected_idx: Option<usize>,
    pub event_rx: EventReceiver,
    pub event_tx: EventSender,

    // Filtering & Multi-Column Sorting
    pub selected_category: CategoryFilter,
    pub search_query: String,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,

    // Real-Time Speed Telemetry (60 rolling 1s samples)
    pub speed_history: Vec<f64>,
    pub last_speed_sample: std::time::Instant,

    // Modals
    pub is_add_url_open: bool,
    pub is_settings_open: bool,
    pub is_details_open: bool,
    pub batch_modal: BatchModalState,
    pub add_url_input: String,
    pub add_url_filename: String,
    pub add_url_connections: usize,
    pub probed_caps: Option<aurora_core::types::ServerCapabilities>,
    pub is_probing: bool,
    pub probe_err: Option<String>,
    pub probe_tx: std::sync::mpsc::Sender<(String, Result<aurora_core::types::ServerCapabilities, String>)>,
    pub probe_rx: std::sync::mpsc::Receiver<(String, Result<aurora_core::types::ServerCapabilities, String>)>,

    // Feedback Toast Notification
    pub toast_message: Option<(String, std::time::Instant)>,

    // Desktop Bridge Command Receiver
    pub command_rx: std::sync::mpsc::Receiver<crate::server::DesktopBridgeCommand>,
}

impl AuroraApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (event_tx, event_rx) = create_event_bus(2048);
        let (probe_tx, probe_rx) = std::sync::mpsc::channel();
        let (cmd_tx_tokio, mut cmd_rx_tokio) = tokio::sync::mpsc::channel::<crate::server::DesktopBridgeCommand>(100);
        let (cmd_tx_std, cmd_rx_std) = std::sync::mpsc::channel();

        // Bridge tokio channel to std channel for GUI thread
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx_tokio.recv().await {
                let _ = cmd_tx_std.send(cmd);
            }
        });

        // Launch embedded Desktop Sync Bridge daemon on 127.0.0.1:28282
        tokio::spawn(async move {
            if let Err(e) = crate::server::start_desktop_server(cmd_tx_tokio).await {
                tracing::warn!("AURORA Desktop Sync Bridge daemon notice: {}", e);
            }
        });

        Self {
            config: EngineConfig::default(),
            downloads: Vec::new(),
            selected_idx: None,
            event_rx,
            event_tx,
            selected_category: CategoryFilter::All,
            search_query: String::new(),
            sort_column: SortColumn::TimeAdded,
            sort_direction: SortDirection::Descending,
            speed_history: vec![0.0; 60],
            last_speed_sample: std::time::Instant::now(),
            is_add_url_open: false,
            is_settings_open: false,
            is_details_open: false,
            batch_modal: BatchModalState::default(),
            add_url_input: String::new(),
            add_url_filename: String::new(),
            add_url_connections: 16,
            probed_caps: None,
            is_probing: false,
            probe_err: None,
            probe_tx,
            probe_rx,
            toast_message: None,
            command_rx: cmd_rx_std,
        }
    }

    // Modal Window Coordination: Opening one modal immediately closes any others
    pub fn open_add_url_modal(&mut self) {
        self.is_add_url_open = true;
        self.batch_modal.is_open = false;
        self.is_settings_open = false;
        self.is_details_open = false;
    }

    pub fn open_batch_modal(&mut self) {
        self.batch_modal.is_open = true;
        self.is_add_url_open = false;
        self.is_settings_open = false;
        self.is_details_open = false;
    }

    pub fn open_settings_modal(&mut self) {
        self.is_settings_open = true;
        self.is_add_url_open = false;
        self.batch_modal.is_open = false;
        self.is_details_open = false;
    }

    pub fn open_details_modal(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
        self.is_details_open = true;
        self.is_add_url_open = false;
        self.batch_modal.is_open = false;
        self.is_settings_open = false;
    }

    pub fn set_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn add_download(&mut self, url_str: &str, custom_filename: Option<&str>, custom_connections: Option<usize>) {
        if let Ok(url) = Url::parse(url_str) {
            let filename = if let Some(cf) = custom_filename {
                if !cf.trim().is_empty() {
                    cf.to_string()
                } else {
                    extract_filename(&url)
                }
            } else {
                extract_filename(&url)
            };

            let dest_path = self.config.download_dir.join(&filename);
            let download_id = DownloadId::new();
            let pause_signal = Arc::new(AtomicBool::new(false));
            let cancel_signal = Arc::new(AtomicBool::new(false));

            let start_time_str = chrono::Local::now().format("%H:%M:%S").to_string();
            let conns = custom_connections.unwrap_or(self.config.initial_connections);

            let item = GuiDownloadItem {
                id: download_id,
                url: url.to_string(),
                filename,
                destination_path: dest_path.to_string_lossy().to_string(),
                total_bytes: None,
                downloaded_bytes: 0,
                current_speed: 0.0,
                peak_speed: 0.0,
                min_speed: 0.0,
                eta: None,
                status: "Downloading".to_string(),
                active_connections: conns,
                http_version: "HTTP/2".to_string(),
                etag: None,
                accepts_ranges: true,
                is_active: true,
                segments: Vec::new(),
                pause_signal: pause_signal.clone(),
                cancel_signal: cancel_signal.clone(),
                requested_pause: false,
                requested_resume: false,
                requested_restart: false,
                requested_open_file: false,
                requested_open_folder: false,
                requested_remove: false,
                requested_delete_file: false,
                requested_reprobe: false,
                requested_inspect: false,
                start_time_str,
                start_instant: std::time::Instant::now(),
                end_time_str: None,
                elapsed_duration: None,
                server_rtt_ms: self.probed_caps.as_ref().and_then(|c| c.rtt_ms),
                server_name: self.probed_caps.as_ref().and_then(|c| c.server_name.clone()),
                health_rating: self.probed_caps.as_ref().and_then(|c| c.health_rating.clone()),
                checksum_input: String::new(),
                checksum_result: None,
            };

            self.downloads.push(item);
            self.selected_idx = Some(self.downloads.len() - 1);

            let mut engine_config = self.config.clone();
            engine_config.initial_connections = conns;
            engine_config.max_connections = conns.max(engine_config.max_connections);
            let tx = self.event_tx.clone();

            // Spawn download engine task on Tokio runtime with the exact same download_id
            tokio::spawn(async move {
                let engine = DownloadEngine::new(url, dest_path, engine_config, tx)
                    .with_download_id(download_id)
                    .with_signals(pause_signal, cancel_signal);
                let _ = engine.run().await;
            });

            self.set_toast(format!("⚡ Download started with {} streams", conns));
        }
    }

    pub fn clear_completed(&mut self) {
        let before_count = self.downloads.len();
        self.downloads.retain(|d| d.status != "Completed");
        let removed = before_count - self.downloads.len();
        if let Some(idx) = self.selected_idx {
            if idx >= self.downloads.len() {
                self.selected_idx = if self.downloads.is_empty() { None } else { Some(self.downloads.len() - 1) };
            }
        }
        if removed > 0 {
            self.set_toast(format!("🧹 Cleared {} completed download(s)", removed));
        }
    }

    pub fn process_events(&mut self, ctx: &Context) {
        while let Ok(cmd) = self.command_rx.try_recv() {
            match cmd {
                crate::server::DesktopBridgeCommand::StartDownload { url, filename, connections } => {
                    self.add_download(&url, filename.as_deref(), connections);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    self.set_toast("📥 Download initiated from Browser Extension");
                }
                crate::server::DesktopBridgeCommand::FocusWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    self.set_toast("⚡ AURORA Desktop Focused from Browser Extension");
                }
            }
        }

        while let Ok(event) = self.event_rx.try_recv() {
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
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
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
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        item.current_speed = smoothed_rate;
                        if smoothed_rate > item.peak_speed {
                            item.peak_speed = smoothed_rate;
                        }
                        if item.start_instant.elapsed() >= Duration::from_secs(2) && smoothed_rate > 50_000.0 {
                            if item.min_speed == 0.0 || smoothed_rate < item.min_speed {
                                item.min_speed = smoothed_rate;
                            }
                        }
                        item.eta = eta;
                        item.downloaded_bytes = downloaded_bytes;
                        if let Some(t) = total_bytes {
                            item.total_bytes = Some(t);
                        }
                    }
                }
                DownloadEvent::SegmentStarted {
                    download_id,
                    segment_id,
                    range,
                    worker_id,
                } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        if let Some(s) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                            s.range = range;
                            s.worker_id = Some(worker_id);
                            s.state = SegmentState::Downloading;
                        } else {
                            let mut seg = Segment::new(segment_id, range);
                            seg.worker_id = Some(worker_id);
                            seg.state = SegmentState::Downloading;
                            item.segments.push(seg);
                        }
                        item.segments.sort_by_key(|s| s.range.start);
                        if (worker_id as usize) > item.active_connections {
                            item.active_connections = worker_id as usize;
                        }
                    }
                }
                DownloadEvent::SegmentProgress {
                    download_id,
                    segment_id,
                    bytes_downloaded,
                    ..
                } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        if let Some(s) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                            s.downloaded = bytes_downloaded;
                            s.state = SegmentState::Downloading;
                        }
                    }
                }
                DownloadEvent::SegmentCompleted {
                    download_id,
                    segment_id,
                    bytes_written,
                } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        if let Some(s) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                            s.downloaded = bytes_written;
                            s.state = SegmentState::Completed;
                        }
                    }
                }
                DownloadEvent::Completed {
                    download_id,
                    elapsed,
                    ..
                } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        item.status = "Completed".to_string();
                        item.is_active = false;
                        item.current_speed = 0.0;
                        item.end_time_str = Some(chrono::Local::now().format("%H:%M:%S").to_string());
                        item.elapsed_duration = Some(elapsed);
                        if let Some(total) = item.total_bytes {
                            item.downloaded_bytes = total;
                        }

                        // Persist to SQLite history database
                        let record = aurora_history::DownloadRecord {
                            id: item.id.0.to_string(),
                            url: item.url.clone(),
                            filename: item.filename.clone(),
                            destination_path: item.destination_path.clone(),
                            file_size: item.total_bytes,
                            downloaded_bytes: item.downloaded_bytes,
                            status: "Completed".to_string(),
                            elapsed_seconds: elapsed.as_secs_f64(),
                            average_speed: if elapsed.as_secs_f64() > 0.0 {
                                item.downloaded_bytes as f64 / elapsed.as_secs_f64()
                            } else {
                                0.0
                            },
                            checksum: item.checksum_result.clone(),
                            created_at: chrono::Utc::now(),
                            completed_at: Some(chrono::Utc::now()),
                        };
                        if let Ok(db) = aurora_history::HistoryDatabase::open(crate::server::get_default_db_path()) {
                            let _ = db.insert_download(&record);
                        }
                    }
                }
                DownloadEvent::StatusChanged {
                    download_id,
                    new_status,
                } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        item.status = new_status.to_string();
                    }
                }
                DownloadEvent::Failed {
                    download_id,
                    error,
                } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        item.status = format!("Error: {}", error);
                        item.is_active = false;
                        item.current_speed = 0.0;
                    }
                }
                _ => {}
            }
        }

        // Process background pre-flight server probe results
        while let Ok((p_url, res)) = self.probe_rx.try_recv() {
            if p_url == self.add_url_input.trim() {
                self.is_probing = false;
                match res {
                    Ok(c) => {
                        self.probed_caps = Some(c);
                        self.probe_err = None;
                    }
                    Err(e) => {
                        self.probe_err = Some(e);
                        self.probed_caps = None;
                    }
                }
            } else {
                // Check if any active download requested a re-probe
                for item in &mut self.downloads {
                    if item.url == p_url {
                        if let Ok(c) = &res {
                            item.server_rtt_ms = c.rtt_ms;
                            item.server_name = c.server_name.clone();
                            item.health_rating = c.health_rating.clone();
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for AuroraApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.process_events(ctx);

        // Sample speed every second for real-time telemetry graph
        let total_speed: f64 = self.downloads.iter().map(|d| d.current_speed).sum();
        if self.last_speed_sample.elapsed() >= Duration::from_secs(1) {
            self.last_speed_sample = std::time::Instant::now();
            self.speed_history.push(total_speed);
            if self.speed_history.len() > 60 {
                self.speed_history.remove(0);
            }
        }

        // ----------------------------------------------------
        // Process User Action Flags (Context Menu & Buttons)
        // ----------------------------------------------------
        let mut to_remove = Vec::new();
        let mut inspect_idx = None;

        for (idx, item) in self.downloads.iter_mut().enumerate() {
            if item.requested_pause {
                item.requested_pause = false;
                item.pause_signal.store(true, Ordering::SeqCst);
                item.is_active = false;
                item.status = "Paused".to_string();
                item.current_speed = 0.0;
            }

            if item.requested_resume {
                item.requested_resume = false;
                item.pause_signal.store(false, Ordering::SeqCst);
                item.cancel_signal.store(false, Ordering::SeqCst);
                item.is_active = true;
                item.status = "Downloading".to_string();
                if let Ok(url) = Url::parse(&item.url) {
                    let dest_path = PathBuf::from(&item.destination_path);
                    let engine_config = self.config.clone();
                    let tx = self.event_tx.clone();
                    let download_id = item.id;
                    let pause_sig = item.pause_signal.clone();
                    let cancel_sig = item.cancel_signal.clone();
                    tokio::spawn(async move {
                        let engine = DownloadEngine::new(url, dest_path, engine_config, tx)
                            .with_download_id(download_id)
                            .with_signals(pause_sig, cancel_sig);
                        let _ = engine.run().await;
                    });
                }
            }

            if item.requested_restart {
                item.requested_restart = false;
                item.cancel_signal.store(true, Ordering::SeqCst);
                let _ = std::fs::remove_file(&item.destination_path);
                let _ = std::fs::remove_file(format!("{}.journal", &item.destination_path));

                item.downloaded_bytes = 0;
                item.current_speed = 0.0;
                item.peak_speed = 0.0;
                item.min_speed = 0.0;
                item.status = "Downloading".to_string();
                item.is_active = true;
                item.start_instant = std::time::Instant::now();
                item.start_time_str = chrono::Local::now().format("%H:%M:%S").to_string();
                item.end_time_str = None;
                item.elapsed_duration = None;
                item.segments.clear();
                item.pause_signal = Arc::new(AtomicBool::new(false));
                item.cancel_signal = Arc::new(AtomicBool::new(false));

                if let Ok(url) = Url::parse(&item.url) {
                    let dest_path = PathBuf::from(&item.destination_path);
                    let engine_config = self.config.clone();
                    let tx = self.event_tx.clone();
                    let download_id = item.id;
                    let pause_sig = item.pause_signal.clone();
                    let cancel_sig = item.cancel_signal.clone();
                    tokio::spawn(async move {
                        let engine = DownloadEngine::new(url, dest_path, engine_config, tx)
                            .with_download_id(download_id)
                            .with_signals(pause_sig, cancel_sig);
                        let _ = engine.run().await;
                    });
                }
            }

            if item.requested_open_folder {
                item.requested_open_folder = false;
                theme::open_file_or_folder(&item.destination_path, true);
            }

            if item.requested_open_file {
                item.requested_open_file = false;
                theme::open_file_or_folder(&item.destination_path, false);
            }

            if item.requested_reprobe {
                item.requested_reprobe = false;
                let url_str = item.url.clone();
                if let Ok(parsed) = Url::parse(&url_str) {
                    let p_tx = self.probe_tx.clone();
                    tokio::spawn(async move {
                        let http_config = aurora_http::client::HttpClientConfig::default();
                        if let Ok(client) = aurora_http::client::create_http_client(&http_config) {
                            let res = aurora_http::probe::probe_url(&client, &parsed).await;
                            let _ = p_tx.send((url_str, res.map_err(|e| e.to_string())));
                        }
                    });
                }
            }

            if item.requested_inspect {
                item.requested_inspect = false;
                inspect_idx = Some(idx);
            }

            if item.requested_remove {
                item.requested_remove = false;
                item.cancel_signal.store(true, Ordering::SeqCst);
                to_remove.push(idx);
            }

            if item.requested_delete_file {
                item.requested_delete_file = false;
                item.cancel_signal.store(true, Ordering::SeqCst);
                let _ = std::fs::remove_file(&item.destination_path);
                let _ = std::fs::remove_file(format!("{}.journal", &item.destination_path));
                to_remove.push(idx);
            }
        }

        if let Some(idx) = inspect_idx {
            self.open_details_modal(idx);
        }

        // Apply removals in reverse index order
        for idx in to_remove.into_iter().rev() {
            if idx < self.downloads.len() {
                self.downloads.remove(idx);
                if self.selected_idx == Some(idx) {
                    self.selected_idx = if self.downloads.is_empty() { None } else { Some(self.downloads.len().saturating_sub(1)) };
                }
            }
        }

        // Request repaint every 100ms for high-precision live graphs & progress
        ctx.request_repaint_after(Duration::from_millis(100));

        // ----------------------------------------------------
        // 1. Top Header Navbar (Matching Extension White Style)
        // ----------------------------------------------------
        TopBottomPanel::top("top_header_panel")
            .frame(egui::Frame::none()
                .fill(theme::COLOR_BG_HEADER)
                .inner_margin(Margin::symmetric(14.0, 10.0))
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Left Brand Block
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("⚡ AURORA")
                                .size(16.0)
                                .strong()
                                .color(theme::COLOR_TEXT_PRIMARY),
                        );
                        pill_badge(ui, "NATIVE", theme::COLOR_ACCENT_BLUE, Color32::WHITE);
                        ui.label(
                            RichText::new("Kaushal IDM v0.2.0")
                                .size(11.0)
                                .color(theme::COLOR_TEXT_SECONDARY),
                        );
                    });

                    ui.add_space(16.0);

                    // Search Filter Field
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔍").size(12.0));
                        ui.add_sized(
                            [180.0, 24.0],
                            egui::TextEdit::singleline(&mut self.search_query).hint_text("Search downloads..."),
                        );
                        if !self.search_query.is_empty() && ui.small_button("✖").clicked() {
                            self.search_query.clear();
                        }
                    });

                    // Right Action Buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if icon_button(ui, "⚙", "Settings").clicked() {
                            self.open_settings_modal();
                        }

                        if icon_button(ui, "📁", "Open Downloads Folder").clicked() {
                            theme::open_file_or_folder(&self.config.download_dir.to_string_lossy(), true);
                        }

                        if icon_button(ui, "🧹", "Clear Completed Downloads").clicked() {
                            self.clear_completed();
                        }

                        if secondary_button(ui, "📦 Batch Add").clicked() {
                            self.open_batch_modal();
                        }

                        if primary_button(ui, "+ Add URL").clicked() {
                            self.open_add_url_modal();
                        }
                    });
                });
            });

        // ----------------------------------------------------
        // 2. Global Stats Bar (Matching Extension #fbfbfd .stats-bar)
        // ----------------------------------------------------
        TopBottomPanel::top("global_stats_bar")
            .frame(egui::Frame::none()
                .fill(theme::COLOR_BG_STATS)
                .inner_margin(Margin::symmetric(16.0, 8.0))
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let active_tasks: usize = self.downloads.iter().filter(|d| d.is_active).count();
                    let total_conns: usize = self.downloads.iter().filter(|d| d.is_active).map(|d| d.active_connections).sum();
                    let total_downloaded: u64 = self.downloads.iter().map(|d| d.downloaded_bytes).sum();

                    // Total Speed
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("TOTAL SPEED:").size(10.5).color(theme::COLOR_TEXT_SECONDARY).strong());
                        let speed_str = format!("{:.2} MB/s", total_speed / (1024.0 * 1024.0));
                        ui.label(RichText::new(speed_str).size(13.0).strong().color(theme::COLOR_ACCENT_BLUE));
                    });

                    ui.separator();

                    // Active Tasks
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ACTIVE TASKS:").size(10.5).color(theme::COLOR_TEXT_SECONDARY).strong());
                        ui.label(RichText::new(format!("{} ({} streams)", active_tasks, total_conns)).size(12.0).strong().color(theme::COLOR_TEXT_PRIMARY));
                    });

                    ui.separator();

                    // Total Downloaded
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("SESSION TRAFFIC:").size(10.5).color(theme::COLOR_TEXT_SECONDARY).strong());
                        ui.label(RichText::new(crate::views::queue_view::format_bytes(total_downloaded)).size(12.0).strong().color(theme::COLOR_TEXT_PRIMARY));
                    });

                    ui.separator();

                    // Engine Badge
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ENGINE:").size(10.5).color(theme::COLOR_TEXT_SECONDARY).strong());
                        pill_badge(ui, &format!("{}", self.config.scheduler_type), theme::COLOR_ACCENT_BLUE, Color32::WHITE);
                    });

                    // Toast message notification
                    if let Some((msg, inst)) = &self.toast_message {
                        if inst.elapsed() < Duration::from_secs(4) {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(msg).small().color(theme::COLOR_SUCCESS_GREEN));
                            });
                        }
                    }
                });
            });

        // ----------------------------------------------------
        // 3. Left Navigation Sidebar
        // ----------------------------------------------------
        SidePanel::left("left_sidebar_panel")
            .resizable(false)
            .default_width(170.0)
            .frame(egui::Frame::none()
                .fill(theme::COLOR_BG_HEADER)
                .inner_margin(Margin::symmetric(10.0, 8.0))
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER)))
            .show(ctx, |ui| {
                render_sidebar(ui, &mut self.selected_category, &self.downloads);
            });

        // ----------------------------------------------------
        // 4. Bottom Panel: Live Telemetry Graph & Selected Segment Map
        // ----------------------------------------------------
        let mut open_inspector_req = false;

        TopBottomPanel::bottom("bottom_status_panel")
            .frame(egui::Frame::none()
                .fill(theme::COLOR_BG_HEADER)
                .inner_margin(Margin::symmetric(14.0, 8.0))
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Left: Live Telemetry Graph
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("📈 Live Bandwidth Telemetry (60s)").strong().size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                        });
                        let peak_speed = self.speed_history.iter().copied().fold(0.0_f64, f64::max);
                        ui.scope(|ui| {
                            ui.set_max_height(48.0);
                            render_speed_graph(ui, &self.speed_history, total_speed, peak_speed);
                        });
                    });

                    ui.separator();

                    // Right: Segment Map of Selected Download
                    ui.vertical(|ui| {
                        if let Some(idx) = self.selected_idx {
                            if let Some(item) = self.downloads.get(idx) {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Segment Allocation:").strong().size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                                    ui.label(RichText::new(&item.filename).strong().size(11.5).color(theme::COLOR_TEXT_PRIMARY));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if secondary_button(ui, "🔍 Inspector").clicked() {
                                            open_inspector_req = true;
                                        }
                                    });
                                });
                                ui.add_space(2.0);
                                render_segment_visualization(ui, &item.segments, item.total_bytes, 16.0);
                            }
                        } else {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("💡 Select any download to inspect live chunk allocation & stream distribution.")
                                    .size(11.0)
                                    .color(theme::COLOR_TEXT_MUTED));
                            });
                        }
                    });
                });
            });

        if open_inspector_req {
            if let Some(idx) = self.selected_idx {
                self.open_details_modal(idx);
            }
        }

        // ----------------------------------------------------
        // 5. Central Download Queue Grid / Cards
        // ----------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.downloads.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(80.0);
                        ui.label(
                            RichText::new("⚡")
                                .size(42.0)
                                .color(theme::COLOR_ACCENT_BLUE),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("No active downloads")
                                .size(16.0)
                                .strong()
                                .color(theme::COLOR_TEXT_PRIMARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Click \"+ Add URL\" or \"📦 Batch Add\" to start accelerated multi-stream downloading.")
                                .size(12.5)
                                .color(theme::COLOR_TEXT_SECONDARY),
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if primary_button(ui, "+ Add URL").clicked() {
                                self.open_add_url_modal();
                            }
                            if secondary_button(ui, "📦 Batch Add").clicked() {
                                self.open_batch_modal();
                            }
                        });
                    });
                });
            } else {
                let visible_indices = filter_and_sort_indices(
                    &self.downloads,
                    self.selected_category,
                    &self.search_query,
                    self.sort_column,
                    self.sort_direction,
                );

                let mut toast_req = None;
                render_queue_table(
                    ui,
                    &mut self.downloads,
                    &visible_indices,
                    &mut self.selected_idx,
                    &mut self.sort_column,
                    &mut self.sort_direction,
                    &mut toast_req,
                );
                if let Some(t) = toast_req {
                    self.set_toast(t);
                }
            }
        });

        // ----------------------------------------------------
        // 6. Modals (+ Add URL, Batch Add, Settings, Details)
        // ----------------------------------------------------
        let any_modal_open = self.is_add_url_open || self.batch_modal.is_open || self.is_settings_open || self.is_details_open;
        let backdrop_alpha = ctx.animate_bool(egui::Id::new("modal_backdrop_anim"), any_modal_open);
        if backdrop_alpha > 0.001 {
            let screen_rect = ctx.screen_rect();
            let dim_color = Color32::from_rgba_premultiplied(0, 0, 0, (backdrop_alpha * 50.0) as u8);
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("modal_backdrop_layer")))
                .rect_filled(screen_rect, Rounding::ZERO, dim_color);
        }

        if self.is_add_url_open {
            let mut is_open = self.is_add_url_open;
            let mut close_requested = false;
            let mut submitted_download: Option<(String, Option<String>)> = None;

            egui::Window::new("⚡ Add New Download & Server Diagnostics")
                .open(&mut is_open)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .pivot(egui::Align2::CENTER_CENTER)
                .collapsible(false)
                .default_size([520.0, 340.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Download URL:").size(12.0).strong().color(theme::COLOR_TEXT_PRIMARY));
                    ui.add_sized(
                        [ui.available_width(), 26.0],
                        egui::TextEdit::singleline(&mut self.add_url_input).hint_text("https://example.com/largefile.zip"),
                    );

                    // Auto-trigger probe when URL is entered or button clicked
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let can_probe = !self.add_url_input.trim().is_empty() && !self.is_probing;
                        if ui.add_enabled(can_probe, egui::Button::new(RichText::new("🔍 Pre-Flight Server Probe").strong())).clicked() {
                            let url_str = self.add_url_input.trim().to_string();
                            if let Ok(parsed) = Url::parse(&url_str) {
                                self.is_probing = true;
                                self.probe_err = None;
                                let p_tx = self.probe_tx.clone();
                                tokio::spawn(async move {
                                    let http_config = aurora_http::client::HttpClientConfig::default();
                                    if let Ok(client) = aurora_http::client::create_http_client(&http_config) {
                                        let res = aurora_http::probe::probe_url(&client, &parsed).await;
                                        let _ = p_tx.send((url_str, res.map_err(|e| e.to_string())));
                                    }
                                });
                            }
                        }

                        if self.is_probing {
                            ui.spinner();
                            ui.label(RichText::new("Probing server latency & BDP...").size(11.5).color(theme::COLOR_ACCENT_BLUE));
                        }
                    });

                    // Server Diagnostic Card (Clean Light Theme)
                    if let Some(caps) = &self.probed_caps {
                        ui.add_space(6.0);
                        egui::Frame::none()
                            .fill(Color32::from_rgb(245, 245, 247))
                            .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(Margin::same(10.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Pre-Flight Bottleneck Diagnosis").strong().color(theme::COLOR_ACCENT_BLUE));
                                    if let Some(rating) = &caps.health_rating {
                                        pill_badge(ui, rating, Color32::from_rgb(230, 235, 248), theme::COLOR_ACCENT_BLUE);
                                    }
                                });
                                ui.separator();

                                let (rtt_color, rtt_icon) = match caps.rtt_ms {
                                    Some(ms) if ms < 45 => (theme::COLOR_SUCCESS_GREEN, "⚡"),
                                    Some(ms) if ms < 120 => (theme::COLOR_WARNING_ORANGE, "🚀"),
                                    _ => (theme::COLOR_DANGER_RED, "🌐"),
                                };

                                let rtt_str = caps.rtt_ms.map(|ms| format!("{} ms", ms)).unwrap_or_else(|| "Unknown".to_string());

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Ping / RTT:").color(theme::COLOR_TEXT_SECONDARY));
                                    ui.label(RichText::new(format!("{} {}", rtt_icon, rtt_str)).strong().color(rtt_color));

                                    ui.separator();
                                    ui.label(RichText::new("Server:").color(theme::COLOR_TEXT_SECONDARY));
                                    ui.label(RichText::new(caps.server_name.as_deref().unwrap_or("Server")).strong().color(theme::COLOR_TEXT_PRIMARY));
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Multi-Stream Ranges:").color(theme::COLOR_TEXT_SECONDARY));
                                    if caps.accepts_ranges {
                                        ui.label(RichText::new("✅ 16 Parallel Range Streams").strong().color(theme::COLOR_SUCCESS_GREEN));
                                    } else {
                                        ui.label(RichText::new("⚠ 1x Single-Stream Only").strong().color(theme::COLOR_WARNING_ORANGE));
                                    }

                                    if let Some(cl) = caps.content_length {
                                        ui.separator();
                                        ui.label(RichText::new("Size:").color(theme::COLOR_TEXT_SECONDARY));
                                        ui.label(RichText::new(crate::views::queue_view::format_bytes(cl)).strong().color(theme::COLOR_TEXT_PRIMARY));
                                    }
                                });
                            });
                    } else if let Some(err) = &self.probe_err {
                        ui.label(RichText::new(format!("⚠ Server Note: {}", err)).small().color(theme::COLOR_WARNING_ORANGE));
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Custom Filename (Optional):").size(12.0).color(theme::COLOR_TEXT_PRIMARY));
                            ui.add_sized(
                                [270.0, 26.0],
                                egui::TextEdit::singleline(&mut self.add_url_filename).hint_text("Auto-detected"),
                            );
                        });

                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Parallel Streams:").size(12.0).color(theme::COLOR_TEXT_PRIMARY));
                            egui::ComboBox::from_id_source("add_modal_conns")
                                .selected_text(format!("{} Streams", self.add_url_connections))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.add_url_connections, 4, "4 Streams");
                                    ui.selectable_value(&mut self.add_url_connections, 8, "8 Streams");
                                    ui.selectable_value(&mut self.add_url_connections, 16, "16 Streams (High-Speed)");
                                    ui.selectable_value(&mut self.add_url_connections, 32, "32 Streams (Extreme)");
                                });
                        });
                    });

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if primary_button(ui, "Start Download").clicked() {
                            let url = self.add_url_input.trim().to_string();
                            let fn_opt = if !self.add_url_filename.trim().is_empty() {
                                Some(self.add_url_filename.trim().to_string())
                            } else {
                                None
                            };

                            if !url.is_empty() {
                                submitted_download = Some((url, fn_opt));
                            }
                        }

                        if secondary_button(ui, "Cancel").clicked() {
                            close_requested = true;
                        }
                    });
                });

            if close_requested {
                is_open = false;
            }

            self.is_add_url_open = is_open;
            if let Some((url, fn_opt)) = submitted_download {
                let conns = self.add_url_connections;
                self.add_download(&url, fn_opt.as_deref(), Some(conns));
                self.add_url_input.clear();
                self.add_url_filename.clear();
                self.probed_caps = None;
                self.probe_err = None;
                self.is_add_url_open = false;
            }
        }

        // Batch Downloader Modal
        let mut batch_submissions = Vec::new();
        render_batch_modal(ctx, &mut self.batch_modal, &mut batch_submissions);
        for (url, fn_opt, conns) in batch_submissions {
            self.add_download(&url, fn_opt.as_deref(), Some(conns));
        }

        // Settings Modal
        if self.is_settings_open {
            render_settings_modal(ctx, &mut self.config, &mut self.is_settings_open);
        }

        // Details / Inspector Modal
        if self.is_details_open {
            if let Some(idx) = self.selected_idx {
                if let Some(item) = self.downloads.get_mut(idx) {
                    render_detail_modal(ctx, item, &mut self.is_details_open);
                }
            }
        }
    }
}

fn extract_filename(url: &Url) -> String {
    url.path_segments()
        .and_then(|mut s| s.next_back())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut decoded = String::new();
            let bytes = s.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'%' && i + 2 < bytes.len() {
                    if let Ok(hex_byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                        decoded.push(hex_byte as char);
                        i += 3;
                        continue;
                    }
                }
                decoded.push(bytes[i] as char);
                i += 1;
            }
            if !decoded.is_empty() {
                decoded
            } else {
                s.to_string()
            }
        })
        .unwrap_or_else(|| "download.bin".to_string())
}
