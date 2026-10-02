use crate::theme::{self, secondary_button};
use egui::{Context, Margin, RichText, Rounding, Stroke, Window};
use url::Url;

pub struct BatchModalState {
    pub is_open: bool,
    pub raw_urls_text: String,
    pub pattern_input: String,
    pub selected_streams: usize,
}

impl Default for BatchModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            raw_urls_text: String::new(),
            pattern_input: String::new(),
            selected_streams: 16,
        }
    }
}

pub fn render_batch_modal(
    ctx: &Context,
    state: &mut BatchModalState,
    submitted_downloads: &mut Vec<(String, Option<String>, usize)>,
) {
    if !state.is_open {
        return;
    }

    let mut open = state.is_open;
    let mut close_req = false;

    Window::new("📦 Batch Multi-URL & Pattern Downloader")
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .pivot(egui::Align2::CENTER_CENTER)
        .collapsible(false)
        .default_size([580.0, 500.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // Tab 1: Direct Multi-URL Paste
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Paste URLs (One Per Line):").strong().size(12.5).color(theme::COLOR_ACCENT_BLUE));
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut state.raw_urls_text)
                            .hint_text("https://example.com/file1.zip\nhttps://example.com/file2.zip\nhttps://example.com/file3.zip")
                            .desired_rows(6)
                            .desired_width(ui.available_width()),
                    );
                });

            ui.add_space(8.0);

            // Tab 2: Pattern URL Expander
            egui::Frame::none()
                .fill(theme::COLOR_BG_CARD)
                .stroke(Stroke::new(1.0_f32, theme::COLOR_BORDER))
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Pattern Range Expander:").strong().size(12.5).color(theme::COLOR_ACCENT_BLUE));
                    ui.label(RichText::new("Use [01-10] or [1-20] in the URL").size(11.0).color(theme::COLOR_TEXT_SECONDARY));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [ui.available_width() - 100.0, 26.0],
                            egui::TextEdit::singleline(&mut state.pattern_input).hint_text("https://cdn.example.com/part_[01-08].mp4"),
                        );
                        if secondary_button(ui, "Expand").clicked() {
                            let expanded = expand_url_pattern(&state.pattern_input);
                            if !expanded.is_empty() {
                                if !state.raw_urls_text.trim().is_empty() {
                                    state.raw_urls_text.push('\n');
                                }
                                state.raw_urls_text.push_str(&expanded.join("\n"));
                                state.pattern_input.clear();
                            }
                        }
                    });
                });

            ui.add_space(8.0);

            // Parse valid URLs
            let lines: Vec<String> = state
                .raw_urls_text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && Url::parse(l).is_ok())
                .collect();

            // Preview & Stream Settings
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Detected Valid URLs: {}", lines.len())).strong().color(if lines.is_empty() { theme::COLOR_TEXT_MUTED } else { theme::COLOR_SUCCESS_GREEN }));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_source("batch_streams_combo")
                        .selected_text(format!("{} Streams", state.selected_streams))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.selected_streams, 4, "4 Streams");
                            ui.selectable_value(&mut state.selected_streams, 8, "8 Streams");
                            ui.selectable_value(&mut state.selected_streams, 16, "16 Streams");
                            ui.selectable_value(&mut state.selected_streams, 32, "32 Streams");
                        });
                    ui.label("Per-Task Concurrency:");
                });
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let can_submit = !lines.is_empty();
                if ui.add_enabled(can_submit, egui::Button::new(RichText::new(format!("🚀 Start {} Batch Downloads", lines.len())).strong().color(egui::Color32::WHITE)).fill(theme::COLOR_ACCENT_BLUE)).clicked() {
                    for url in lines {
                        submitted_downloads.push((url, None, state.selected_streams));
                    }
                    state.raw_urls_text.clear();
                    close_req = true;
                }

                if secondary_button(ui, "Cancel").clicked() {
                    close_req = true;
                }
            });
        });

    if close_req {
        state.is_open = false;
    } else {
        state.is_open = open;
    }
}

/// Expands patterns like `[01-10]` or `[1-15]` in a URL string
fn expand_url_pattern(pattern: &str) -> Vec<String> {
    if let (Some(start_idx), Some(end_idx)) = (pattern.find('['), pattern.find(']')) {
        if start_idx < end_idx {
            let prefix = &pattern[..start_idx];
            let bracket_content = &pattern[start_idx + 1..end_idx];
            let suffix = &pattern[end_idx + 1..];

            if let Some((start_s, end_s)) = bracket_content.split_once('-') {
                if let (Ok(start_num), Ok(end_num)) = (start_s.trim().parse::<u64>(), end_s.trim().parse::<u64>()) {
                    let pad_len = if start_s.starts_with('0') && start_s.len() > 1 {
                        start_s.len()
                    } else {
                        1
                    };

                    let mut result = Vec::new();
                    let (min, max) = if start_num <= end_num { (start_num, end_num) } else { (end_num, start_num) };
                    let capped_max = max.min(min + 100); // safety cap to 100 items

                    for n in min..=capped_max {
                        let num_str = format!("{:0width$}", n, width = pad_len);
                        result.push(format!("{}{}{}", prefix, num_str, suffix));
                    }
                    return result;
                }
            }
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_url_pattern_padded() {
        let pattern = "https://example.com/file_[01-05].zip";
        let urls = expand_url_pattern(pattern);
        assert_eq!(
            urls,
            vec![
                "https://example.com/file_01.zip",
                "https://example.com/file_02.zip",
                "https://example.com/file_03.zip",
                "https://example.com/file_04.zip",
                "https://example.com/file_05.zip",
            ]
        );
    }

    #[test]
    fn test_expand_url_pattern_unpadded() {
        let pattern = "https://example.com/episode_[1-3].mp4";
        let urls = expand_url_pattern(pattern);
        assert_eq!(
            urls,
            vec![
                "https://example.com/episode_1.mp4",
                "https://example.com/episode_2.mp4",
                "https://example.com/episode_3.mp4",
            ]
        );
    }
}

