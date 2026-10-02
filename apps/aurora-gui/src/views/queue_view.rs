use crate::app::GuiDownloadItem;
use crate::sorting::{SortColumn, SortDirection};
use crate::theme::{self, COLOR_ACCENT_BLUE, COLOR_DANGER_RED, COLOR_SUCCESS_GREEN, COLOR_WARNING_ORANGE};
use egui::{Margin, RichText, Rounding, ScrollArea, Stroke, Ui};

pub fn render_queue_table(
    ui: &mut Ui,
    items: &mut [GuiDownloadItem],
    visible_indices: &[usize],
    selected_idx: &mut Option<usize>,
    sort_col: &mut SortColumn,
    sort_dir: &mut SortDirection,
    toast_tx: &mut Option<String>,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(4.0);

            // Table Header Bar with Interactive Sort Buttons
            egui::Frame::none()
                .fill(theme::COLOR_BG_HEADER)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(14.0, 4.0))
                .show(ui, |ui| {
                    egui::Grid::new("download_queue_header_grid")
                        .min_col_width(70.0)
                        .spacing([14.0, 2.0])
                        .show(ui, |ui| {
                            sort_header_button(ui, "Name / Server", SortColumn::Name, sort_col, sort_dir);
                            sort_header_button(ui, "Size", SortColumn::Size, sort_col, sort_dir);
                            sort_header_button(ui, "Status / Progress", SortColumn::Status, sort_col, sort_dir);
                            sort_header_button(ui, "Download Speed", SortColumn::Speed, sort_col, sort_dir);
                            sort_header_button(ui, "Time & Took", SortColumn::TimeTook, sort_col, sort_dir);
                            sort_header_button(ui, "Remaining ETA", SortColumn::Eta, sort_col, sort_dir);
                            ui.label(RichText::new("Actions").strong().size(11.5).color(theme::COLOR_TEXT_SECONDARY));
                            ui.end_row();
                        });
                });

            ui.add_space(6.0);

            if visible_indices.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("No downloads match the active category / search filter.")
                            .size(13.0)
                            .color(theme::COLOR_TEXT_MUTED),
                    );
                });
                return;
            }

            // Download Rows Container (Iterating through filtered & sorted indices)
            for &idx in visible_indices {
                if idx >= items.len() {
                    continue;
                }
                let item = &mut items[idx];
                let is_selected = *selected_idx == Some(idx);
                let bg_color = if is_selected {
                    theme::COLOR_BG_CARD_SELECTED
                } else {
                    theme::COLOR_BG_CARD
                };

                let border_stroke = if is_selected {
                    Stroke::new(1.0_f32, theme::COLOR_BORDER_FOCUS)
                } else {
                    Stroke::new(1.0_f32, theme::COLOR_BORDER)
                };

                let card_frame = egui::Frame::none()
                    .fill(bg_color)
                    .stroke(border_stroke)
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::symmetric(14.0, 10.0));

                let card_response = card_frame.show(ui, |ui| {
                    egui::Grid::new(format!("row_grid_{}", idx))
                        .min_col_width(70.0)
                        .spacing([14.0, 6.0])
                        .show(ui, |ui| {
                            // Column 1: Icon + Filename + Server Diagnostics
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let icon = get_file_icon(&item.filename);
                                    ui.label(RichText::new(icon).size(15.0));

                                    let file_label = ui.selectable_label(
                                        is_selected,
                                        RichText::new(&item.filename)
                                            .strong()
                                            .size(13.0)
                                            .color(if is_selected {
                                                COLOR_ACCENT_BLUE
                                            } else {
                                                theme::COLOR_TEXT_PRIMARY
                                            }),
                                    );
                                    if file_label.clicked() {
                                        *selected_idx = Some(idx);
                                    }
                                });

                                // Server Diagnostic Badge on Row
                                if let Some(rtt) = item.server_rtt_ms {
                                    let (rtt_color, ping_icon) = if rtt < 45 {
                                        (COLOR_SUCCESS_GREEN, "⚡")
                                    } else if rtt < 120 {
                                        (COLOR_WARNING_ORANGE, "🚀")
                                    } else {
                                        (COLOR_DANGER_RED, "🌐")
                                    };
                                    let s_name = item.server_name.as_deref().unwrap_or("Server");
                                    let streams_tag = if item.accepts_ranges { "16x Range" } else { "1x Single" };
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("{} {}ms • {} ({})", ping_icon, rtt, s_name, streams_tag))
                                                .size(11.0)
                                                .color(rtt_color),
                                        );
                                    });
                                }
                            });

                            // Column 2: Size (Downloaded / Total)
                            let size_text = match item.total_bytes {
                                Some(total) => {
                                    if item.status == "Completed" {
                                        format_bytes(total)
                                    } else {
                                        format!("{}/{}", format_bytes_live(item.downloaded_bytes), format_bytes(total))
                                    }
                                }
                                None => {
                                    if item.is_active {
                                        format!("{}/Probing...", format_bytes_live(item.downloaded_bytes))
                                    } else if item.downloaded_bytes > 0 {
                                        format_bytes_live(item.downloaded_bytes)
                                    } else {
                                        "Unknown".to_string()
                                    }
                                }
                            };
                            ui.label(
                                RichText::new(size_text)
                                    .size(12.0)
                                    .color(theme::COLOR_TEXT_PRIMARY),
                            );

                            // Column 3: Status (Progress Bar + Percentage)
                            let progress_fraction = match item.total_bytes {
                                Some(t) if t > 0 => {
                                    (item.downloaded_bytes as f32 / t as f32).clamp(0.0, 1.0)
                                }
                                _ => {
                                    if item.status == "Completed" {
                                        1.0
                                    } else {
                                        0.0
                                    }
                                }
                            };

                            let progress_text = if item.status == "Completed" {
                                "100%".to_string()
                            } else {
                                let pct = (progress_fraction * 100.0).clamp(0.0, 99.9);
                                if pct >= 99.0 {
                                    format!("{:.1}%", pct)
                                } else if pct >= 10.0 {
                                    format!("{:.0}%", pct.floor())
                                } else {
                                    format!("{:.1}%", pct)
                                }
                            };

                            ui.horizontal(|ui| {
                                let bar_color = if item.status == "Completed" {
                                    COLOR_SUCCESS_GREEN
                                } else if item.status == "Finalizing" {
                                    COLOR_SUCCESS_GREEN
                                } else if item.status == "Paused" {
                                    COLOR_WARNING_ORANGE
                                } else if item.status.starts_with("Error") {
                                    COLOR_DANGER_RED
                                } else {
                                    COLOR_ACCENT_BLUE
                                };

                                let bar = egui::ProgressBar::new(progress_fraction)
                                    .fill(bar_color)
                                    .rounding(Rounding::same(4.0));

                                ui.add_sized([120.0, 14.0], bar);
                                ui.label(
                                    RichText::new(progress_text)
                                        .strong()
                                        .size(11.5)
                                        .color(theme::COLOR_TEXT_PRIMARY),
                                );
                            });

                            // Column 4: Download Speed (Current / Peak)
                            ui.vertical(|ui| {
                                let speed_text = if item.is_active && item.current_speed > 0.0 {
                                    format!("{}/s", format_speed(item.current_speed))
                                } else if item.status == "Completed" || item.status == "Finalizing" {
                                    "0 B/s".to_string()
                                } else {
                                    "-".to_string()
                                };

                                ui.label(
                                    RichText::new(speed_text)
                                        .strong()
                                        .size(12.5)
                                        .color(if item.is_active && item.current_speed > 0.0 {
                                            theme::COLOR_ACCENT_BLUE
                                        } else {
                                            theme::COLOR_TEXT_MUTED
                                        }),
                                );

                                if item.is_active && item.peak_speed > 0.0 {
                                    ui.label(
                                        RichText::new(format!("Peak: {}/s", format_speed(item.peak_speed)))
                                            .size(10.0)
                                            .color(theme::COLOR_TEXT_MUTED),
                                    );
                                }
                            });

                            // Column 5: Time & Duration
                            let time_summary = if item.status == "Completed" {
                                let end_str = item.end_time_str.as_deref().unwrap_or("-");
                                let took_str = match item.elapsed_duration {
                                    Some(d) => format_duration(d),
                                    None => format_duration(item.start_instant.elapsed()),
                                };
                                format!("{} → {}\nTook {}", item.start_time_str, end_str, took_str)
                            } else if item.is_active {
                                let elapsed = item.start_instant.elapsed();
                                format!("Started {}\nTook {}", item.start_time_str, format_duration(elapsed))
                            } else if item.status == "Paused" {
                                format!("Started {}\n(Paused)", item.start_time_str)
                            } else {
                                format!("Started {}", item.start_time_str)
                            };

                            ui.label(
                                RichText::new(time_summary)
                                    .size(11.0)
                                    .color(if item.status == "Completed" {
                                        COLOR_SUCCESS_GREEN
                                    } else {
                                        theme::COLOR_TEXT_SECONDARY
                                    }),
                            );

                            // Column 6: Remaining ETA
                            let remaining_text = if item.status == "Completed" {
                                "Completed".to_string()
                            } else if item.status == "Finalizing" {
                                "Finalizing...".to_string()
                            } else if item.status == "Paused" {
                                format!("Paused ({})", format_bytes(item.downloaded_bytes))
                            } else if item.status.starts_with("Error") {
                                item.status.clone()
                            } else if let Some(eta) = item.eta {
                                if item.is_active && item.current_speed > 0.0 {
                                    let secs = eta.as_secs();
                                    if secs == 0 {
                                        "Finishing...".to_string()
                                    } else {
                                        format!("ETA {}", format_duration(eta))
                                    }
                                } else {
                                    "Calculating...".to_string()
                                }
                            } else if item.is_active {
                                "Connecting...".to_string()
                            } else {
                                "Queued".to_string()
                            };

                            ui.label(
                                RichText::new(remaining_text)
                                    .strong()
                                    .size(11.5)
                                    .color(if item.status == "Completed" {
                                        COLOR_SUCCESS_GREEN
                                    } else if item.status == "Paused" {
                                        COLOR_WARNING_ORANGE
                                    } else {
                                        theme::COLOR_TEXT_PRIMARY
                                    }),
                            );

                            // Column 7: Quick Actions (Pause/Resume + Inspector + Menu)
                            ui.horizontal(|ui| {
                                if item.is_active {
                                    if ui.button(RichText::new("⏸").size(12.0).color(theme::COLOR_TEXT_PRIMARY)).on_hover_text("Pause download").clicked() {
                                        item.requested_pause = true;
                                    }
                                } else if item.status == "Paused" || item.status.starts_with("Error") {
                                    if ui.button(RichText::new("▶").size(12.0).color(COLOR_WARNING_ORANGE)).on_hover_text("Resume download").clicked() {
                                        item.requested_resume = true;
                                    }
                                }

                                if ui.button(RichText::new("🔍").size(12.0)).on_hover_text("View Inspector").clicked() {
                                    *selected_idx = Some(idx);
                                    item.requested_inspect = true;
                                }

                                let menu_btn = ui.button(RichText::new("•••").size(12.0).color(theme::COLOR_TEXT_SECONDARY)).on_hover_text("Right-click for options");
                                menu_btn.context_menu(|ui| {
                                    render_context_menu_items(ui, item, selected_idx, idx, toast_tx);
                                });
                            });

                            ui.end_row();
                        });
                });

                // Attach Right-Click Context Menu to the entire Card Response
                card_response.response.context_menu(|ui| {
                    render_context_menu_items(ui, item, selected_idx, idx, toast_tx);
                });

                ui.add_space(4.0);
            }
        });
}

fn sort_header_button(
    ui: &mut Ui,
    label: &str,
    col: SortColumn,
    current_col: &mut SortColumn,
    current_dir: &mut SortDirection,
) {
    let is_current = *current_col == col && *current_dir != SortDirection::None;
    let arrow_str = if is_current { current_dir.arrow() } else { "" };
    let text = format!("{}{}", label, arrow_str);

    let rich = if is_current {
        RichText::new(text).strong().size(11.5).color(theme::COLOR_ACCENT_BLUE)
    } else {
        RichText::new(text).strong().size(11.5).color(theme::COLOR_TEXT_SECONDARY)
    };

    if ui.button(rich).clicked() {
        if *current_col == col {
            *current_dir = current_dir.toggle();
        } else {
            *current_col = col;
            *current_dir = SortDirection::Ascending;
        }
    }
}

/// Helper to render the interactive Right-Click Context Menu
fn render_context_menu_items(
    ui: &mut Ui,
    item: &mut GuiDownloadItem,
    selected_idx: &mut Option<usize>,
    idx: usize,
    toast_tx: &mut Option<String>,
) {
    ui.set_min_width(200.0);

    ui.label(RichText::new(&item.filename).strong().size(12.5).color(theme::COLOR_TEXT_PRIMARY));
    ui.separator();

    if item.is_active {
        if ui.button(RichText::new("⏸  Pause Download").size(12.0)).clicked() {
            item.requested_pause = true;
            ui.close_menu();
        }
    } else if item.status == "Paused" || item.status.starts_with("Error") {
        if ui.button(RichText::new("▶  Resume Download").size(12.0)).clicked() {
            item.requested_resume = true;
            ui.close_menu();
        }
    }

    if ui.button(RichText::new("🔄  Restart / Redownload").size(12.0)).clicked() {
        item.requested_restart = true;
        ui.close_menu();
    }

    ui.separator();

    if ui.button(RichText::new("📁  Open Containing Folder").size(12.0)).clicked() {
        item.requested_open_folder = true;
        ui.close_menu();
    }

    if item.status == "Completed" {
        if ui.button(RichText::new("📄  Open Downloaded File").size(12.0)).clicked() {
            item.requested_open_file = true;
            ui.close_menu();
        }
    }

    ui.separator();

    if ui.button(RichText::new("📋  Copy Download URL").size(12.0)).clicked() {
        ui.output_mut(|o| o.copied_text = item.url.clone());
        *toast_tx = Some("✓ Copied Download URL to clipboard".to_string());
        ui.close_menu();
    }

    if ui.button(RichText::new("📋  Copy Local File Path").size(12.0)).clicked() {
        ui.output_mut(|o| o.copied_text = item.destination_path.clone());
        *toast_tx = Some("✓ Copied File Path to clipboard".to_string());
        ui.close_menu();
    }

    if ui.button(RichText::new("⚡  Re-probe Server & Latency").size(12.0)).clicked() {
        item.requested_reprobe = true;
        ui.close_menu();
    }

    ui.separator();

    if ui.button(RichText::new("🔍  View Details & Segment Map").size(12.0)).clicked() {
        *selected_idx = Some(idx);
        item.requested_inspect = true;
        ui.close_menu();
    }

    ui.separator();

    if ui.button(RichText::new("🗑  Remove from List").size(12.0).color(theme::COLOR_WARNING_ORANGE)).clicked() {
        item.requested_remove = true;
        ui.close_menu();
    }

    if ui.button(RichText::new("🗑️  Delete File from Disk").size(12.0).color(theme::COLOR_DANGER_RED)).clicked() {
        item.requested_delete_file = true;
        ui.close_menu();
    }
}

fn get_file_icon(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if lower.ends_with(".zip") || lower.ends_with(".tar") || lower.ends_with(".gz") || lower.ends_with(".7z") || lower.ends_with(".rar") || lower.ends_with(".iso") {
        "📦"
    } else if lower.ends_with(".mp4") || lower.ends_with(".mkv") || lower.ends_with(".avi") || lower.ends_with(".mov") || lower.ends_with(".webm") {
        "🎬"
    } else if lower.ends_with(".mp3") || lower.ends_with(".flac") || lower.ends_with(".wav") || lower.ends_with(".aac") {
        "🎵"
    } else if lower.ends_with(".pdf") || lower.ends_with(".doc") || lower.ends_with(".docx") || lower.ends_with(".txt") {
        "📄"
    } else if lower.ends_with(".exe") || lower.ends_with(".deb") || lower.ends_with(".rpm") || lower.ends_with(".dmg") || lower.ends_with(".bin") {
        "⚙️"
    } else {
        "📄"
    }
}

pub fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.2} GB", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.2} MB", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1} KB", bytes_per_sec / KB)
    } else {
        format!("{:.0} B", bytes_per_sec)
    }
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", (bytes as f64 / MB as f64).round())
    } else if bytes >= KB {
        format!("{:.0} KB", (bytes as f64 / KB as f64).round())
    } else {
        format!("{} B", bytes)
    }
}

fn format_bytes_live(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= 100 * MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_duration(d: std::time::Duration) -> String {
    let total_secs = d.as_secs();
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {:02}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}
