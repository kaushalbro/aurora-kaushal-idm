use crate::app::GuiDownloadItem;
use crate::theme::{self, primary_button, secondary_button, COLOR_ACCENT_BLUE, COLOR_SUCCESS_GREEN, COLOR_WARNING_ORANGE};
use crate::views::segment_view::render_segment_visualization;
use egui::{Context, Margin, RichText, Rounding, ScrollArea, Stroke, Window};

pub fn render_detail_modal(ctx: &Context, item: &mut GuiDownloadItem, is_open: &mut bool) {
    let mut open = *is_open;
    let mut close_req = false;

    Window::new(format!("🔍 Inspector — {}", item.filename))
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .pivot(egui::Align2::CENTER_CENTER)
        .collapsible(false)
        .default_size([700.0, 580.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // Metadata Card
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Stream & Metadata Overview").strong().size(13.0).color(COLOR_ACCENT_BLUE));
                    ui.separator();

                    egui::Grid::new("details_meta_grid")
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("URL:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            ui.label(RichText::new(&item.url).color(theme::COLOR_TEXT_PRIMARY));
                            ui.end_row();

                            ui.label(RichText::new("Destination:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            ui.label(RichText::new(&item.destination_path).color(theme::COLOR_TEXT_PRIMARY));
                            ui.end_row();

                            ui.label(RichText::new("Status:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            ui.label(RichText::new(&item.status).strong().color(if item.status == "Completed" { COLOR_SUCCESS_GREEN } else { COLOR_ACCENT_BLUE }));
                            ui.end_row();

                            ui.label(RichText::new("Downloaded / Size:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            let total_str = match item.total_bytes {
                                Some(t) => format!("{:.2} MB / {:.2} MB ({:.1}%)", item.downloaded_bytes as f64 / (1024.0 * 1024.0), t as f64 / (1024.0 * 1024.0), (item.downloaded_bytes as f64 / t as f64 * 100.0).clamp(0.0, 100.0)),
                                None => format!("{:.2} MB / Unknown", item.downloaded_bytes as f64 / (1024.0 * 1024.0)),
                            };
                            ui.label(RichText::new(total_str).color(theme::COLOR_TEXT_PRIMARY));
                            ui.end_row();

                            ui.label(RichText::new("Current / Peak Speed:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            ui.label(RichText::new(format!("{:.2} MB/s (Peak: {:.2} MB/s)", item.current_speed / (1024.0 * 1024.0), item.peak_speed / (1024.0 * 1024.0))).color(theme::COLOR_ACCENT_CYAN));
                            ui.end_row();

                            ui.label(RichText::new("Server Latency & Host:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            let rtt_str = item.server_rtt_ms.map(|ms| format!("{} ms", ms)).unwrap_or_else(|| "Unknown".to_string());
                            let host_str = item.server_name.as_deref().unwrap_or("Server");
                            ui.label(RichText::new(format!("⚡ {} • {}", rtt_str, host_str)).color(theme::COLOR_TEXT_PRIMARY));
                            ui.end_row();

                            ui.label(RichText::new("HTTP Protocol:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            ui.label(RichText::new(&item.http_version).color(theme::COLOR_TEXT_PRIMARY));
                            ui.end_row();

                            ui.label(RichText::new("ETag Validator:").strong().color(theme::COLOR_TEXT_SECONDARY));
                            ui.label(RichText::new(item.etag.as_deref().unwrap_or("None")).color(theme::COLOR_TEXT_MUTED));
                            ui.end_row();
                        });
                });

            ui.add_space(8.0);

            // Cryptographic Checksum Verifier Card
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Cryptographic Hash & Integrity Verifier").strong().size(13.0).color(COLOR_ACCENT_BLUE));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Expected Checksum:").size(11.5).color(theme::COLOR_TEXT_PRIMARY));
                        ui.add_sized([ui.available_width() - 130.0, 24.0], egui::TextEdit::singleline(&mut item.checksum_input).hint_text("Paste expected SHA-256 or BLAKE3 hex..."));

                        if secondary_button(ui, "Verify Hash").clicked() {
                            let path = std::path::Path::new(&item.destination_path);
                            if path.exists() {
                                let exp = item.checksum_input.trim();
                                if exp.is_empty() {
                                    item.checksum_result = Some("⚠ Please paste an expected checksum string".to_string());
                                } else {
                                    let algo = if exp.len() == 64 {
                                        aurora_storage::integrity::HashAlgorithm::Sha256
                                    } else {
                                        aurora_storage::integrity::HashAlgorithm::Blake3
                                    };
                                    match aurora_storage::integrity::verify_file_checksum(path, exp, algo) {
                                        Ok(true) => item.checksum_result = Some("✅ Exact Cryptographic Match Verified!".to_string()),
                                        Ok(false) => item.checksum_result = Some("❌ Checksum Mismatch (File Corrupted or Altered)".to_string()),
                                        Err(e) => item.checksum_result = Some(format!("⚠ Error: {}", e)),
                                    }
                                }
                            } else {
                                item.checksum_result = Some("⚠ File not yet complete on disk".to_string());
                            }
                        }
                    });

                    if let Some(res) = &item.checksum_result {
                        ui.add_space(4.0);
                        let color = if res.starts_with('✅') {
                            COLOR_SUCCESS_GREEN
                        } else if res.starts_with('❌') {
                            theme::COLOR_DANGER_RED
                        } else {
                            COLOR_WARNING_ORANGE
                        };
                        ui.label(RichText::new(res).strong().size(12.0).color(color));
                    }
                });

            ui.add_space(8.0);

            // Segment Visualization Card
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Live Chunk Allocations").strong().size(13.0).color(COLOR_ACCENT_BLUE));
                    ui.add_space(4.0);
                    render_segment_visualization(ui, &item.segments, item.total_bytes, 20.0);
                });

            ui.add_space(8.0);

            // Active Segments Table Card
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Worker Stream Telemetry").strong().size(13.0).color(COLOR_ACCENT_BLUE));
                    ui.separator();

                    ScrollArea::vertical()
                        .max_height(140.0)
                        .show(ui, |ui| {
                            egui::Grid::new("segments_table_grid")
                                .striped(true)
                                .spacing([14.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("ID").strong().size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                                    ui.label(RichText::new("Byte Range").strong().size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                                    ui.label(RichText::new("Progress").strong().size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                                    ui.label(RichText::new("Worker").strong().size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                                    ui.label(RichText::new("State").strong().size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                                    ui.end_row();

                                    for seg in &item.segments {
                                        ui.label(RichText::new(format!("#{}", seg.id)).size(11.0));
                                        ui.label(RichText::new(format!("{}-{}", seg.range.start, seg.range.end)).size(11.0));
                                        let pct = if seg.range.len() > 0 {
                                            (seg.downloaded as f64 / seg.range.len() as f64 * 100.0).clamp(0.0, 100.0)
                                        } else {
                                            0.0
                                        };
                                        ui.label(RichText::new(format!("{:.1}%", pct)).size(11.0).color(COLOR_ACCENT_BLUE));
                                        ui.label(
                                            RichText::new(seg.worker_id.map(|w| format!("W-{}", w)).unwrap_or_else(|| "-".to_string())).size(11.0),
                                        );
                                        ui.label(RichText::new(format!("{:?}", seg.state)).size(11.0).color(if seg.state == aurora_core::types::SegmentState::Completed { COLOR_SUCCESS_GREEN } else { COLOR_WARNING_ORANGE }));
                                        ui.end_row();
                                    }
                                });
                        });
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if primary_button(ui, "Close").clicked() {
                    close_req = true;
                }
            });
        });

    if close_req {
        *is_open = false;
    } else {
        *is_open = open;
    }
}
