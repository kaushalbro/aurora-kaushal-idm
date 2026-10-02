use aurora_core::config::{EngineConfig, SchedulerType};
use egui::{Context, Window};
use std::time::Duration;

pub fn render_settings_modal(
    ctx: &Context,
    config: &mut EngineConfig,
    is_open: &mut bool,
) {
    Window::new("Settings — AURORA Download Engine")
        .open(is_open)
        .default_size([500.0, 420.0])
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("General Settings");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Download Directory:");
                let mut path_str = config.download_dir.to_string_lossy().to_string();
                if ui.text_edit_singleline(&mut path_str).changed() {
                    config.download_dir = std::path::PathBuf::from(path_str);
                }
            });

            ui.add(
                egui::Slider::new(&mut config.max_simultaneous_downloads, 1..=10)
                    .text("Max Concurrent Downloads"),
            );

            ui.add_space(12.0);
            ui.heading("Engine & Scheduler Settings");
            ui.separator();

            egui::ComboBox::from_label("Scheduling Algorithm")
                .selected_text(format!("{}", config.scheduler_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut config.scheduler_type,
                        SchedulerType::AuroraEct,
                        "AURORA Expected Completion Time (ECT)",
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

            ui.checkbox(&mut config.adaptive_concurrency, "Enable Adaptive Concurrency Control");

            ui.add(
                egui::Slider::new(&mut config.initial_connections, 1..=32)
                    .text("Initial Connections"),
            );
            ui.add(
                egui::Slider::new(&mut config.max_connections, 1..=64)
                    .text("Max Connections Limit"),
            );

            let mut seg_secs = config.target_segment_duration.as_secs_f64();
            if ui
                .add(egui::Slider::new(&mut seg_secs, 1.0..=15.0).text("Target Segment Duration (sec)"))
                .changed()
            {
                config.target_segment_duration = Duration::from_secs_f64(seg_secs);
            }

            ui.add_space(12.0);
            ui.heading("Storage & Backpressure");
            ui.separator();

            let mut queue_mb = config.disk_queue_limit_bytes / (1024 * 1024);
            if ui
                .add(egui::Slider::new(&mut queue_mb, 16..=512).text("Disk In-Memory Queue Limit (MB)"))
                .changed()
            {
                config.disk_queue_limit_bytes = queue_mb * 1024 * 1024;
            }

            ui.checkbox(&mut config.tail_hedging, "Enable Tail Straggler Hedging (final 1%)");
        });
}
