use crate::theme::{self, primary_button};
use aurora_core::config::{EngineConfig, SchedulerType};
use egui::{Context, Margin, RichText, Rounding, Stroke, Window};
use std::time::Duration;

pub fn render_settings_modal(
    ctx: &Context,
    config: &mut EngineConfig,
    is_open: &mut bool,
) {
    let mut open = *is_open;
    let mut close_req = false;

    Window::new("⚙ Configuration & Preferences")
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .pivot(egui::Align2::CENTER_CENTER)
        .collapsible(false)
        .default_size([540.0, 480.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // Section 1: General Settings
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("General Settings").strong().size(13.0).color(theme::COLOR_ACCENT_BLUE));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Download Directory:").color(theme::COLOR_TEXT_PRIMARY));
                        let mut path_str = config.download_dir.to_string_lossy().to_string();
                        if ui.text_edit_singleline(&mut path_str).changed() {
                            config.download_dir = std::path::PathBuf::from(path_str);
                        }
                    });

                    ui.add(
                        egui::Slider::new(&mut config.max_simultaneous_downloads, 1..=10)
                            .text("Max Concurrent Tasks"),
                    );
                });

            ui.add_space(8.0);

            // Section 2: Engine & Scheduler Settings
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Engine & Multi-Stream Scheduling").strong().size(13.0).color(theme::COLOR_ACCENT_BLUE));
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Scheduler Algorithm:").color(theme::COLOR_TEXT_PRIMARY));
                        egui::ComboBox::from_id_source("sched_combo")
                            .selected_text(format!("{}", config.scheduler_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut config.scheduler_type,
                                    SchedulerType::AuroraEct,
                                    "AURORA ECT (Adaptive Completion Time)",
                                );
                                ui.selectable_value(
                                    &mut config.scheduler_type,
                                    SchedulerType::LargestSegment,
                                    "Largest-Segment Splitting (IDM-style)",
                                );
                                ui.selectable_value(
                                    &mut config.scheduler_type,
                                    SchedulerType::Fixed,
                                    "Fixed Segmentation",
                                );
                                ui.selectable_value(
                                    &mut config.scheduler_type,
                                    SchedulerType::SingleStream,
                                    "Single Stream (Baseline)",
                                );
                            });
                    });

                    ui.checkbox(&mut config.adaptive_concurrency, "Enable Adaptive BDP Concurrency Control");

                    ui.add(
                        egui::Slider::new(&mut config.initial_connections, 1..=32)
                            .text("Initial Connections (16 Recommended)"),
                    );
                    ui.add(
                        egui::Slider::new(&mut config.max_connections, 1..=64)
                            .text("Max Connections Limit (32 Extreme)"),
                    );

                    let mut seg_secs = config.target_segment_duration.as_secs_f64();
                    if ui
                        .add(egui::Slider::new(&mut seg_secs, 1.0..=15.0).text("Target Segment Duration (sec)"))
                        .changed()
                    {
                        config.target_segment_duration = Duration::from_secs_f64(seg_secs);
                    }
                });

            ui.add_space(8.0);

            // Section 3: Storage & Backpressure
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Storage & Straggler Mitigation").strong().size(13.0).color(theme::COLOR_ACCENT_BLUE));
                    ui.separator();

                    let mut queue_mb = config.disk_queue_limit_bytes / (1024 * 1024);
                    if ui
                        .add(egui::Slider::new(&mut queue_mb, 16..=512).text("In-Memory Write Buffer (MB)"))
                        .changed()
                    {
                        config.disk_queue_limit_bytes = queue_mb * 1024 * 1024;
                    }

                    ui.checkbox(&mut config.tail_hedging, "Enable Endgame BitTorrent Micro-Splitting & Tail Hedging");
                });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if primary_button(ui, "Done").clicked() {
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
