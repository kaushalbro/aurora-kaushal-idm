use crate::views::detail_view::render_detail_modal;
use crate::views::queue_view::render_queue_table;
use crate::views::segment_view::render_segment_visualization;
use crate::views::settings_view::render_settings_modal;
use aurora_core::config::EngineConfig;
use aurora_core::events::{create_event_bus, DownloadEvent, EventReceiver, EventSender};
use aurora_core::types::{DownloadId, Segment, SegmentState};
use aurora_scheduler::engine::DownloadEngine;
use egui::{Color32, Context, RichText, TopBottomPanel};
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
    pub requested_pause: bool,
    pub requested_resume: bool,
    pub start_time_str: String,
    pub start_instant: std::time::Instant,
    pub end_time_str: Option<String>,
    pub elapsed_duration: Option<Duration>,
    pub server_rtt_ms: Option<u64>,
    pub server_name: Option<String>,
    pub health_rating: Option<String>,
}

pub struct AuroraApp {
    pub config: EngineConfig,
    pub downloads: Vec<GuiDownloadItem>,
    pub selected_idx: Option<usize>,
    pub event_rx: EventReceiver,
    pub event_tx: EventSender,

    // Modals
    pub is_add_url_open: bool,
    pub is_settings_open: bool,
    pub is_details_open: bool,
    pub add_url_input: String,
    pub add_url_filename: String,
    pub probed_caps: Option<aurora_core::types::ServerCapabilities>,
    pub is_probing: bool,
    pub probe_err: Option<String>,
    pub probe_tx: std::sync::mpsc::Sender<(String, Result<aurora_core::types::ServerCapabilities, String>)>,
    pub probe_rx: std::sync::mpsc::Receiver<(String, Result<aurora_core::types::ServerCapabilities, String>)>,
}

impl AuroraApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (event_tx, event_rx) = create_event_bus(2048);
        let (probe_tx, probe_rx) = std::sync::mpsc::channel();
        Self {
            config: EngineConfig::default(),
            downloads: Vec::new(),
            selected_idx: None,
            event_rx,
            event_tx,
            is_add_url_open: false,
            is_settings_open: false,
            is_details_open: false,
            add_url_input: String::new(),
            add_url_filename: String::new(),
            probed_caps: None,
            is_probing: false,
            probe_err: None,
            probe_tx,
            probe_rx,
        }
    }

    pub fn add_download(&mut self, url_str: &str, custom_filename: Option<&str>) {
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
                active_connections: self.config.initial_connections,
                http_version: "HTTP/2".to_string(),
                etag: None,
                accepts_ranges: true,
                is_active: true,
                segments: Vec::new(),
                pause_signal: pause_signal.clone(),
                cancel_signal: cancel_signal.clone(),
                requested_pause: false,
                requested_resume: false,
                start_time_str,
                start_instant: std::time::Instant::now(),
                end_time_str: None,
                elapsed_duration: None,
                server_rtt_ms: self.probed_caps.as_ref().and_then(|c| c.rtt_ms),
                server_name: self.probed_caps.as_ref().and_then(|c| c.server_name.clone()),
                health_rating: self.probed_caps.as_ref().and_then(|c| c.health_rating.clone()),
            };

            self.downloads.push(item);
            self.selected_idx = Some(self.downloads.len() - 1);

            let engine_config = self.config.clone();
            let tx = self.event_tx.clone();

            // Spawn download engine task on Tokio runtime with the exact same download_id
            tokio::spawn(async move {
                let engine = DownloadEngine::new(url, dest_path, engine_config, tx)
                    .with_download_id(download_id)
                    .with_signals(pause_signal, cancel_signal);
                let _ = engine.run().await;
            });
        }
    }

    pub fn process_events(&mut self) {
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
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        if let Some(seg) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                            seg.state = SegmentState::Downloading;
                            seg.worker_id = Some(worker_id);
                        } else {
                            let mut new_seg = Segment::new(segment_id, range);
                            new_seg.state = SegmentState::Downloading;
                            new_seg.worker_id = Some(worker_id);
                            item.segments.push(new_seg);
                        }
                    }
                }
                DownloadEvent::SegmentCompleted {
                    download_id,
                    segment_id,
                    ..
                } => {
                    if let Some(item) = self.downloads.iter_mut().find(|d| d.id == download_id) {
                        if let Some(seg) = item.segments.iter_mut().find(|s| s.id == segment_id) {
                            seg.state = SegmentState::Completed;
                            seg.downloaded = seg.range.len();
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
            }
        }
    }
}

impl eframe::App for AuroraApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.process_events();

        // Handle user pause and resume button requests
        for item in &mut self.downloads {
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
        }

        // Request repaint every 100ms for responsive progress and throughput updates
        ctx.request_repaint_after(Duration::from_millis(100));

        // 1. Top Action Toolbar
        TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("⚡ AURORA Kaushal")
                        .strong()
                        .color(Color32::from_rgb(100, 180, 255)),
                );
                ui.separator();

                if ui
                    .button(RichText::new("+ Add URL").strong())
                    .clicked()
                {
                    self.is_add_url_open = true;
                }

                if let Some(idx) = self.selected_idx {
                    if let Some(item) = self.downloads.get_mut(idx) {
                        if item.is_active {
                            if ui.button("⏸ Pause").clicked() {
                                item.requested_pause = true;
                            }
                        } else if item.status == "Paused" {
                            if ui.button("▶ Resume").clicked() {
                                item.requested_resume = true;
                            }
                        }
                    }
                }

                if ui.button("🗑 Remove").clicked() {
                    if let Some(idx) = self.selected_idx {
                        if idx < self.downloads.len() {
                            let item = &self.downloads[idx];
                            item.cancel_signal.store(true, Ordering::SeqCst);
                            self.downloads.remove(idx);
                            self.selected_idx = None;
                        }
                    }
                }

                ui.separator();

                if ui.button("⚙ Settings").clicked() {
                    self.is_settings_open = true;
                }

                if let Some(idx) = self.selected_idx {
                    if self.downloads.get(idx).is_some() {
                        if ui.button("🔍 Details").clicked() {
                            self.is_details_open = true;
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("AURORA Kaushal IDM v0.1.0")
                            .color(Color32::from_rgb(140, 140, 160))
                            .small(),
                    );
                });
            });
            ui.add_space(4.0);
        });

        // 2. Bottom Status Bar & Selected Segment Visualizer
        TopBottomPanel::bottom("bottom_status_panel").show(ctx, |ui| {
            ui.add_space(4.0);

            // If an item is selected, show its live segment visualization
            if let Some(idx) = self.selected_idx {
                if let Some(item) = self.downloads.get(idx) {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Segment Map:").strong().small());
                        ui.label(RichText::new(&item.filename).small());
                    });
                    render_segment_visualization(ui, &item.segments, item.total_bytes, 16.0);
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                let total_speed: f64 = self.downloads.iter().map(|d| d.current_speed).sum();
                let total_conns: usize = self
                    .downloads
                    .iter()
                    .filter(|d| d.is_active)
                    .map(|d| d.active_connections)
                    .sum();

                ui.label(format!(
                    "Total Speed: {:.2} MB/s | Active Conns: {} | Scheduler: {}",
                    total_speed / (1024.0 * 1024.0),
                    total_conns,
                    self.config.scheduler_type
                ));
            });
            ui.add_space(4.0);
        });

        // 3. Central Download Queue Grid
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.downloads.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("No downloads yet.\nClick '+ Add URL' to begin downloading.")
                            .color(Color32::from_rgb(140, 140, 160)),
                    );
                });
            } else {
                render_queue_table(ui, &mut self.downloads, &mut self.selected_idx);
            }
        });

        // Modals
        if self.is_add_url_open {
            let mut is_open = self.is_add_url_open;
            let mut close_requested = false;
            let mut submitted_download: Option<(String, Option<String>)> = None;

            egui::Window::new("⚡ Add New Download & Server Check")
                .open(&mut is_open)
                .default_size([480.0, 280.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Download URL:").strong());
                    ui.text_edit_singleline(&mut self.add_url_input);

                    ui.horizontal(|ui| {
                        let can_probe = !self.add_url_input.trim().is_empty() && !self.is_probing;
                        if ui.add_enabled(can_probe, egui::Button::new(RichText::new("🔍 Check Server & Latency").strong())).clicked() {
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
                            ui.label(RichText::new("Probing server latency & capabilities...").color(Color32::from_rgb(90, 200, 250)));
                        }
                    });

                    // Server Diagnostic Card
                    if let Some(caps) = &self.probed_caps {
                        ui.add_space(4.0);
                        egui::Frame::none()
                            .fill(Color32::from_rgb(22, 28, 40))
                            .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(45, 60, 90)))
                            .rounding(6.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Server Performance & Health Check:").strong().color(Color32::from_rgb(100, 180, 255)));
                                });
                                ui.separator();

                                let rtt_str = match caps.rtt_ms {
                                    Some(ms) => format!("{} ms", ms),
                                    None => "Unknown".to_string(),
                                };
                                let (rtt_color, rtt_icon) = match caps.rtt_ms {
                                    Some(ms) if ms < 45 => (Color32::from_rgb(76, 217, 100), "⚡"),
                                    Some(ms) if ms < 120 => (Color32::from_rgb(255, 214, 10), "🚀"),
                                    _ => (Color32::from_rgb(255, 149, 0), "🌐"),
                                };

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Ping / Latency:").color(Color32::from_rgb(180, 185, 195)));
                                    ui.label(RichText::new(format!("{} {}", rtt_icon, rtt_str)).strong().color(rtt_color));

                                    ui.separator();
                                    ui.label(RichText::new("Server:").color(Color32::from_rgb(180, 185, 195)));
                                    ui.label(RichText::new(caps.server_name.as_deref().unwrap_or("Web Server")).strong().color(Color32::WHITE));
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Protocol:").color(Color32::from_rgb(180, 185, 195)));
                                    ui.label(RichText::new(&caps.http_version).strong().color(Color32::WHITE));

                                    ui.separator();
                                    ui.label(RichText::new("File Size:").color(Color32::from_rgb(180, 185, 195)));
                                    let size_str = caps.content_length.map(crate::views::queue_view::format_bytes).unwrap_or_else(|| "Unknown".to_string());
                                    ui.label(RichText::new(size_str).strong().color(Color32::from_rgb(76, 217, 100)));
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Multi-Stream:").color(Color32::from_rgb(180, 185, 195)));
                                    if caps.accepts_ranges {
                                        ui.label(RichText::new("✅ Supported (16 Parallel Connections)").color(Color32::from_rgb(76, 217, 100)).strong());
                                    } else {
                                        ui.label(RichText::new("⚠ Single-Stream Only (No Range Support)").color(Color32::from_rgb(255, 149, 0)).strong());
                                    }
                                });

                                if let Some(rating) = &caps.health_rating {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Diagnosis:").color(Color32::from_rgb(180, 185, 195)));
                                        ui.label(RichText::new(rating).small().color(Color32::from_rgb(200, 210, 230)));
                                    });
                                }
                            });
                    } else if let Some(err) = &self.probe_err {
                        ui.label(RichText::new(format!("⚠ Probe Note: {}", err)).small().color(Color32::from_rgb(255, 179, 0)));
                    }

                    ui.add_space(6.0);
                    ui.label("Filename (Optional):");
                    ui.text_edit_singleline(&mut self.add_url_filename);

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Start Download").strong()).clicked() {
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

                        if ui.button("Cancel").clicked() {
                            close_requested = true;
                        }
                    });
                });

            if close_requested {
                is_open = false;
            }

            self.is_add_url_open = is_open;
            if let Some((url, fn_opt)) = submitted_download {
                self.add_download(&url, fn_opt.as_deref());
                self.add_url_input.clear();
                self.add_url_filename.clear();
                self.probed_caps = None;
                self.probe_err = None;
                self.is_add_url_open = false;
            }
        }

        if self.is_settings_open {
            render_settings_modal(ctx, &mut self.config, &mut self.is_settings_open);
        }

        if self.is_details_open {
            if let Some(idx) = self.selected_idx {
                if let Some(item) = self.downloads.get(idx) {
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
