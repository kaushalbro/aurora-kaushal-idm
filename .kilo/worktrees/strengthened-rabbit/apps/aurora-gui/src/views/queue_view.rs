use crate::app::GuiDownloadItem;
use egui::{Color32, RichText, ScrollArea, Ui};

pub fn render_queue_table(
    ui: &mut Ui,
    items: &mut [GuiDownloadItem],
    selected_idx: &mut Option<usize>,
) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new("download_queue_grid")
                .striped(true)
                .min_col_width(75.0)
                .spacing([16.0, 12.0])
                .show(ui, |ui| {
                    // Header Row
                    ui.label(RichText::new("Name").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.label(RichText::new("Size").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.label(RichText::new("Status").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.label(RichText::new("Download (Cur / Peak / Low)").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.label(RichText::new("Upload").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.label(RichText::new("Time (Start / End / Took)").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.label(RichText::new("Remaining").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.label(RichText::new("Actions").strong().color(Color32::from_rgb(180, 185, 200)));
                    ui.end_row();

                    for (idx, item) in items.iter_mut().enumerate() {
                        let is_selected = *selected_idx == Some(idx);

                        // Column 1: Control Icon + File Name + Server Health Badge
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if item.is_active {
                                    if ui.small_button(RichText::new("⏸").color(Color32::WHITE))
                                        .on_hover_text("Pause download")
                                        .clicked()
                                    {
                                        item.requested_pause = true;
                                    }
                                } else if item.status == "Paused" {
                                    if ui.small_button(RichText::new("▶").color(Color32::from_rgb(255, 179, 0)))
                                        .on_hover_text("Resume download")
                                        .clicked()
                                    {
                                        item.requested_resume = true;
                                    }
                                } else if item.status == "Completed" {
                                    ui.label(RichText::new("✓").color(Color32::from_rgb(76, 217, 100)).strong());
                                } else {
                                    ui.label(RichText::new("•").color(Color32::from_rgb(160, 165, 175)));
                                }

                                let file_label = ui.selectable_label(
                                    is_selected,
                                    RichText::new(&item.filename)
                                        .strong()
                                        .color(if is_selected {
                                            Color32::WHITE
                                        } else {
                                            Color32::from_rgb(220, 225, 235)
                                        }),
                                );
                                if file_label.clicked() {
                                    *selected_idx = Some(idx);
                                }
                            });

                            // Server Diagnostic Badge on the row
                            if let Some(rtt) = item.server_rtt_ms {
                                let (rtt_color, ping_icon) = if rtt < 45 {
                                    (Color32::from_rgb(76, 217, 100), "⚡")
                                } else if rtt < 120 {
                                    (Color32::from_rgb(255, 214, 10), "🚀")
                                } else {
                                    (Color32::from_rgb(255, 149, 0), "🌐")
                                };
                                let s_name = item.server_name.as_deref().unwrap_or("Server");
                                let streams_tag = if item.accepts_ranges { "16x Range" } else { "1x Single" };
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{} {}ms • {} ({})", ping_icon, rtt, s_name, streams_tag))
                                            .small()
                                            .color(rtt_color),
                                    );
                                });
                            }
                        });

                        // Column 2: Live Downloaded / Total Size
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
                                .color(Color32::from_rgb(200, 205, 215)),
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
                                Color32::from_rgb(76, 217, 100)
                            } else if item.status == "Finalizing" {
                                Color32::from_rgb(52, 199, 89)
                            } else if item.status == "Paused" {
                                Color32::from_rgb(255, 179, 0)
                            } else if item.status.starts_with("Error") {
                                Color32::from_rgb(255, 69, 58)
                            } else {
                                Color32::from_rgb(90, 200, 250)
                            };

                            let bar = egui::ProgressBar::new(progress_fraction)
                                .fill(bar_color);

                            ui.add_sized([130.0, 14.0], bar);
                            ui.label(
                                RichText::new(progress_text)
                                    .strong()
                                    .color(Color32::from_rgb(230, 235, 245)),
                            );
                        });

                        // Column 4: Download Speed (Current, Peak, Low)
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
                                    .color(if item.is_active && item.current_speed > 0.0 {
                                        Color32::from_rgb(90, 200, 250)
                                    } else {
                                        Color32::from_rgb(160, 165, 175)
                                    }),
                            );

                            if item.peak_speed > 0.0 {
                                let min_str = if item.min_speed > 0.0 {
                                    format!("{}/s", format_speed(item.min_speed))
                                } else {
                                    "-".to_string()
                                };
                                ui.label(
                                    RichText::new(format!("▲ {}/s  ▼ {}", format_speed(item.peak_speed), min_str))
                                        .small()
                                        .color(Color32::from_rgb(140, 145, 160)),
                                );
                            }
                        });

                        // Column 5: Upload Speed (Client stats)
                        let upload_text = "0 B/s";
                        ui.label(
                            RichText::new(upload_text)
                                .color(Color32::from_rgb(160, 165, 175)),
                        );

                        // Column 6: Time (Start / End / Took)
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
                                .color(if item.status == "Completed" {
                                    Color32::from_rgb(76, 217, 100)
                                } else {
                                    Color32::from_rgb(200, 205, 220)
                                }),
                        );

                        // Column 7: Remaining / Added Status
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
                                    format!("Remaining {}", format_duration(eta))
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
                                .color(if item.status == "Completed" {
                                    Color32::from_rgb(76, 217, 100)
                                } else if item.status == "Finalizing" {
                                    Color32::from_rgb(52, 199, 89)
                                } else if item.status == "Paused" {
                                    Color32::from_rgb(255, 179, 0)
                                } else {
                                    Color32::from_rgb(220, 225, 235)
                                }),
                        );

                        // Column 8: Quick Action Buttons
                        ui.horizontal(|ui| {
                            if item.is_active {
                                if ui.small_button(RichText::new("⏸").color(Color32::WHITE)).on_hover_text("Pause").clicked() {
                                    item.requested_pause = true;
                                }
                            } else if item.status == "Paused" {
                                if ui.small_button(RichText::new("▶").color(Color32::from_rgb(255, 179, 0))).on_hover_text("Resume").clicked() {
                                    item.requested_resume = true;
                                }
                            }
                            if ui.small_button("🔍").on_hover_text("Inspect Details").clicked() {
                                *selected_idx = Some(idx);
                            }
                        });

                        ui.end_row();
                    }
                });
        });
}

fn format_speed(bytes_per_sec: f64) -> String {
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
