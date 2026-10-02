use crate::app::GuiDownloadItem;
use crate::sorting::CategoryFilter;
use crate::theme::{self, pill_badge, COLOR_SUCCESS_GREEN};
use egui::{Color32, Margin, RichText, Rounding, Stroke, Ui};

pub fn render_sidebar(
    ui: &mut Ui,
    selected_category: &mut CategoryFilter,
    items: &[GuiDownloadItem],
) {
    ui.vertical(|ui| {
        ui.add_space(6.0);

        // Sidebar Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("📁 CATEGORIES")
                    .size(11.0)
                    .strong()
                    .color(theme::COLOR_TEXT_SECONDARY),
            );
        });
        ui.add_space(4.0);

        // State Filters Group
        let state_filters = [
            CategoryFilter::All,
            CategoryFilter::Downloading,
            CategoryFilter::Completed,
            CategoryFilter::Paused,
            CategoryFilter::Error,
        ];

        for filter in state_filters {
            let is_active = *selected_category == filter;
            let count = items.iter().filter(|it| filter.matches(it)).count();

            let (bg_color, text_color) = if is_active {
                (theme::COLOR_BG_CARD_SELECTED, theme::COLOR_ACCENT_BLUE)
            } else {
                (Color32::TRANSPARENT, theme::COLOR_TEXT_PRIMARY)
            };

            let stroke = if is_active {
                Stroke::new(1.0_f32, theme::COLOR_BORDER_FOCUS)
            } else {
                Stroke::NONE
            };

            let frame = egui::Frame::none()
                .fill(bg_color)
                .stroke(stroke)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(8.0, 6.0));

            let res = frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(filter.icon()).size(12.0));
                    ui.label(
                        RichText::new(filter.label())
                            .size(12.0)
                            .strong()
                            .color(text_color),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (badge_bg, badge_fg) = match filter {
                            CategoryFilter::Downloading if count > 0 => (theme::COLOR_ACCENT_BLUE, Color32::WHITE),
                            CategoryFilter::Completed if count > 0 => (theme::COLOR_SUCCESS_GREEN_DIM, COLOR_SUCCESS_GREEN),
                            CategoryFilter::Error if count > 0 => (theme::COLOR_DANGER_RED_DIM, theme::COLOR_DANGER_RED),
                            _ => (theme::COLOR_BG_CARD, theme::COLOR_TEXT_SECONDARY),
                        };
                        pill_badge(ui, &count.to_string(), badge_bg, badge_fg);
                    });
                });
            });

            if res.response.interact(egui::Sense::click()).clicked() {
                *selected_category = filter;
            }
            ui.add_space(2.0);
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(6.0);

        // File Type Categories Group
        ui.label(
            RichText::new("📦 FILE TYPES")
                .size(11.0)
                .strong()
                .color(theme::COLOR_TEXT_SECONDARY),
        );
        ui.add_space(4.0);

        let type_filters = [
            CategoryFilter::Compressed,
            CategoryFilter::Video,
            CategoryFilter::Audio,
            CategoryFilter::Documents,
            CategoryFilter::Programs,
        ];

        for filter in type_filters {
            let is_active = *selected_category == filter;
            let count = items.iter().filter(|it| filter.matches(it)).count();

            let (bg_color, text_color) = if is_active {
                (theme::COLOR_BG_CARD_SELECTED, theme::COLOR_ACCENT_BLUE)
            } else {
                (Color32::TRANSPARENT, theme::COLOR_TEXT_PRIMARY)
            };

            let stroke = if is_active {
                Stroke::new(1.0_f32, theme::COLOR_BORDER_FOCUS)
            } else {
                Stroke::NONE
            };

            let frame = egui::Frame::none()
                .fill(bg_color)
                .stroke(stroke)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(8.0, 6.0));

            let res = frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(filter.icon()).size(12.0));
                    ui.label(
                        RichText::new(filter.label())
                            .size(12.0)
                            .color(text_color),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if count > 0 {
                            pill_badge(ui, &count.to_string(), theme::COLOR_BG_CARD, theme::COLOR_TEXT_SECONDARY);
                        }
                    });
                });
            });

            if res.response.interact(egui::Sense::click()).clicked() {
                *selected_category = filter;
            }
            ui.add_space(2.0);
        }
    });
}
