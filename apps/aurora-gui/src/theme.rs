use egui::{Color32, Context, Margin, Response, RichText, Rounding, Stroke, Ui, Vec2, Visuals};
use std::process::Command;

// ==========================================
// Extension-Matching Light macOS Color Palette Constants
// ==========================================
pub const COLOR_ACCENT_BLUE: Color32 = Color32::from_rgb(0, 122, 255);       // #007aff
pub const COLOR_ACCENT_BLUE_HOVER: Color32 = Color32::from_rgb(0, 102, 214); // #0066d6
pub const COLOR_ACCENT_BLUE_ACTIVE: Color32 = Color32::from_rgb(0, 88, 186); // #0058ba
pub const COLOR_ACCENT_CYAN: Color32 = Color32::from_rgb(0, 122, 255);      // #007aff

pub const COLOR_SUCCESS_GREEN: Color32 = Color32::from_rgb(52, 199, 89);     // #34c759
pub const COLOR_SUCCESS_GREEN_DIM: Color32 = Color32::from_rgb(235, 250, 240); // rgba(52, 199, 89, 0.12)
pub const COLOR_WARNING_ORANGE: Color32 = Color32::from_rgb(255, 149, 0);    // #ff9500
pub const COLOR_WARNING_ORANGE_DIM: Color32 = Color32::from_rgb(255, 248, 235); // rgba(255, 149, 0, 0.12)
pub const COLOR_DANGER_RED: Color32 = Color32::from_rgb(255, 59, 48);        // #ff3b30
pub const COLOR_DANGER_RED_DIM: Color32 = Color32::from_rgb(255, 240, 240); // rgba(255, 59, 48, 0.12)

pub const COLOR_BG_CANVAS: Color32 = Color32::from_rgb(245, 245, 247);       // #f5f5f7 Extension Light Canvas
pub const COLOR_BG_HEADER: Color32 = Color32::from_rgb(255, 255, 255);       // #ffffff Navbar surface
pub const COLOR_BG_STATS: Color32 = Color32::from_rgb(251, 251, 253);        // #fbfbfd Stats bar background
pub const COLOR_BG_CARD: Color32 = Color32::from_rgb(255, 255, 255);         // #ffffff Row/Card surface
pub const COLOR_BG_CARD_HOVER: Color32 = Color32::from_rgb(250, 250, 252);   // #fafafc Hovered row card
pub const COLOR_BG_CARD_SELECTED: Color32 = Color32::from_rgb(232, 240, 254);// #e8f0fe macOS Selected Blue
pub const COLOR_BG_INPUT: Color32 = Color32::from_rgb(255, 255, 255);        // #ffffff Form input surface
pub const COLOR_TRACK_BG: Color32 = Color32::from_rgb(229, 229, 234);        // #e5e5ea Progress bar track

pub const COLOR_BORDER: Color32 = Color32::from_rgb(229, 229, 234);          // #e5e5ea (rgba(0, 0, 0, 0.08))
pub const COLOR_BORDER_STRONG: Color32 = Color32::from_rgb(209, 209, 214);   // #d1d1d6 (rgba(0, 0, 0, 0.16))
pub const COLOR_BORDER_FOCUS: Color32 = Color32::from_rgb(0, 122, 255);      // #007aff

pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(29, 29, 31);       // #1d1d1f Dark macOS Text
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(134, 134, 139);  // #86868b
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(174, 174, 178);      // #aeaeb2

/// Configure egui global visuals to match the extension's clean, sleek macOS white / light styling
pub fn configure_visuals(ctx: &Context) {
    let mut visuals = Visuals::light();

    visuals.panel_fill = COLOR_BG_CANVAS;
    visuals.window_fill = Color32::WHITE;
    visuals.extreme_bg_color = COLOR_BG_INPUT;
    visuals.faint_bg_color = COLOR_BG_STATS;

    visuals.window_rounding = Rounding::same(10.0);
    visuals.menu_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.window_shadow.blur = 16.0;
    visuals.window_shadow.spread = 2.0;
    visuals.window_shadow.color = Color32::from_rgba_premultiplied(0, 0, 0, 30);

    // Non-interactive widgets
    visuals.widgets.noninteractive.bg_fill = COLOR_BG_CARD;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

    // Inactive widgets (buttons, inputs)
    visuals.widgets.inactive.bg_fill = Color32::WHITE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::same(6.0);

    // Hovered widgets
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(242, 242, 247);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    // Active (pressed) widgets
    visuals.widgets.active.bg_fill = COLOR_ACCENT_BLUE_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, COLOR_ACCENT_BLUE);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::same(6.0);

    // Open dropdowns / context menus
    visuals.widgets.open.bg_fill = Color32::WHITE;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.widgets.open.rounding = Rounding::same(8.0);

    visuals.selection.bg_fill = COLOR_BG_CARD_SELECTED;
    visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_ACCENT_BLUE);
    visuals.hyperlink_color = COLOR_ACCENT_BLUE;
    visuals.override_text_color = Some(COLOR_TEXT_PRIMARY);

    ctx.set_visuals(visuals);
}

// ==========================================
// Custom UI Widget Helpers
// ==========================================

/// Primary Action Button (Extension `.btn-primary` macOS style)
pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .strong()
            .size(12.5)
            .color(Color32::WHITE),
    )
    .fill(COLOR_ACCENT_BLUE)
    .stroke(Stroke::NONE)
    .rounding(Rounding::same(6.0))
    .min_size(Vec2::new(0.0, 26.0));

    ui.add(button)
}

/// Secondary Outlined Button (Extension `.btn-secondary` style)
pub fn secondary_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .size(12.0)
            .color(COLOR_TEXT_PRIMARY),
    )
    .fill(Color32::WHITE)
    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
    .rounding(Rounding::same(6.0))
    .min_size(Vec2::new(0.0, 26.0));

    ui.add(button)
}

/// Danger Action Button (Red style)
pub fn danger_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .size(12.0)
            .color(Color32::WHITE),
    )
    .fill(COLOR_DANGER_RED)
    .stroke(Stroke::NONE)
    .rounding(Rounding::same(6.0))
    .min_size(Vec2::new(0.0, 26.0));

    ui.add(button)
}

/// Minimal Icon / Toolbar Button
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(icon)
            .size(13.0)
            .color(COLOR_TEXT_PRIMARY),
    )
    .fill(Color32::WHITE)
    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
    .rounding(Rounding::same(6.0))
    .min_size(Vec2::new(28.0, 26.0));

    ui.add(button).on_hover_text(tooltip)
}

/// Renders a rounded pill badge (e.g., `NATIVE`, `16x Range`, `⚡ 18ms`)
pub fn pill_badge(ui: &mut Ui, text: &str, bg: Color32, fg: Color32) {
    let frame = egui::Frame::none()
        .fill(bg)
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::symmetric(6.0, 2.5));

    frame.show(ui, |ui| {
        ui.label(
            RichText::new(text)
                .size(10.5)
                .strong()
                .color(fg),
        );
    });
}

/// Opens a file or its parent directory in the OS default file manager / application
pub fn open_file_or_folder(path_str: &str, open_containing_folder: bool) {
    let path = std::path::Path::new(path_str);
    let target = if open_containing_folder {
        if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        }
    } else {
        path
    };

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(target).spawn();
    }

    #[cfg(target_os = "windows")]
    {
        if open_containing_folder && path.is_file() {
            let _ = Command::new("explorer").arg(format!("/select,{}", path.display())).spawn();
        } else {
            let _ = Command::new("explorer").arg(target).spawn();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if open_containing_folder && path.is_file() {
            let _ = Command::new("open").arg("-R").arg(path).spawn();
        } else {
            let _ = Command::new("open").arg(target).spawn();
        }
    }
}
