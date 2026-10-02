use crate::app::GuiDownloadItem;
use crate::views::segment_view::render_segment_visualization;
use egui::{Context, RichText, ScrollArea, Window};

pub fn render_detail_modal(ctx: &Context, item: &GuiDownloadItem, is_open: &mut bool) {
    Window::new(format!("Download Details — {}", item.filename))
        .open(is_open)
        .default_size([650.0, 480.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading(&item.filename);
            ui.separator();

            // Metadata summary
            egui::Grid::new("details_meta_grid")
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("URL:").strong());
                    ui.label(&item.url);
                    ui.end_row();

                    ui.label(RichText::new("Destination:").strong());
                    ui.label(&item.destination_path);
                    ui.end_row();

                    ui.label(RichText::new("Status:").strong());
                    ui.label(&item.status);
                    ui.end_row();

                    ui.label(RichText::new("Size / Downloaded:").strong());
                    let total_str = match item.total_bytes {
                        Some(t) => format!("{:.2} MB / {:.2} MB ({:.1}%)", item.downloaded_bytes as f64 / (1024.0 * 1024.0), t as f64 / (1024.0 * 1024.0), (item.downloaded_bytes as f64 / t as f64 * 100.0).clamp(0.0, 100.0)),
                        None => format!("{:.2} MB / Unknown", item.downloaded_bytes as f64 / (1024.0 * 1024.0)),
                    };
                    ui.label(total_str);
                    ui.end_row();

                    ui.label(RichText::new("Start Time:").strong());
                    ui.label(&item.start_time_str);
                    ui.end_row();

                    ui.label(RichText::new("End Time:").strong());
                    ui.label(item.end_time_str.as_deref().unwrap_or("In progress..."));
                    ui.end_row();

                    ui.label(RichText::new("Total Time Taken:").strong());
                    let took_str = match item.elapsed_duration {
                        Some(d) => format!("{:.2?}s", d.as_secs_f64()),
                        None => format!("{:.2?}s (live)", item.start_instant.elapsed().as_secs_f64()),
                    };
                    ui.label(took_str);
                    ui.end_row();

                    ui.label(RichText::new("Current Speed:").strong());
                    ui.label(format!("{:.2} MB/s", item.current_speed / (1024.0 * 1024.0)));
                    ui.end_row();

                    ui.label(RichText::new("Peak Speed:").strong());
                    ui.label(format!("{:.2} MB/s", item.peak_speed / (1024.0 * 1024.0)));
                    ui.end_row();

                    ui.label(RichText::new("Lowest Speed:").strong());
                    let low_str = if item.min_speed > 0.0 {
                        format!("{:.2} MB/s", item.min_speed / (1024.0 * 1024.0))
                    } else {
                        "-".to_string()
                    };
                    ui.label(low_str);
                    ui.end_row();

                    ui.label(RichText::new("HTTP Protocol:").strong());
                    ui.label(&item.http_version);
                    ui.end_row();

                    ui.label(RichText::new("ETag Validator:").strong());
                    ui.label(item.etag.as_deref().unwrap_or("None"));
                    ui.end_row();

                    ui.label(RichText::new("Range Requests:").strong());
                    ui.label(if item.accepts_ranges {
                        "Supported (Multi-segment enabled)"
                    } else {
                        "Unsupported (Single-stream only)"
                    });
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.label(RichText::new("Segment Visualization:").strong());
            render_segment_visualization(ui, &item.segments, item.total_bytes, 24.0);

            ui.add_space(10.0);
            ui.label(RichText::new("Active Segments & Workers:").strong());

            ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    egui::Grid::new("segments_table_grid")
                        .striped(true)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("ID").strong());
                            ui.label(RichText::new("Range").strong());
                            ui.label(RichText::new("Downloaded").strong());
                            ui.label(RichText::new("Progress").strong());
                            ui.label(RichText::new("Worker").strong());
                            ui.label(RichText::new("State").strong());
                            ui.end_row();

                            for seg in &item.segments {
                                ui.label(format!("#{}", seg.id));
                                ui.label(format!("{}-{}", seg.range.start, seg.range.end));
                                ui.label(format!("{}/{}", seg.downloaded, seg.range.len()));
                                ui.label(format!("{:.1}%", seg.progress_fraction() * 100.0));
                                ui.label(
                                    seg.worker_id
                                        .map(|w| format!("W-{}", w))
                                        .unwrap_or_else(|| "-".to_string()),
                                );
                                ui.label(format!("{:?}", seg.state));
                                ui.end_row();
                            }
                        });
                });
        });
}
