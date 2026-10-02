use egui::{Color32, Context, Margin, Response, RichText, Rounding, Stroke, Ui, Vec2, Visuals};
use std::process::Command;

// ==========================================
// Modern High-Contrast Cyber Midnight Dark Palette
// ==========================================
pub const COLOR_ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246);       // #3b82f6 Vibrant Electric Blue
pub const COLOR_ACCENT_BLUE_HOVER: Color32 = Color32::from_rgb(96, 165, 250); // #60a5fa
pub const COLOR_ACCENT_BLUE_ACTIVE: Color32 = Color32::from_rgb(37, 99, 235); // #2563eb
pub const COLOR_ACCENT_CYAN: Color32 = Color32::from_rgb(6, 182, 212);        // #06b6d4 Neon Cyan

pub const COLOR_SUCCESS_GREEN: Color32 = Color32::from_rgb(16, 185, 129);     // #10b981 Emerald Green
pub const COLOR_SUCCESS_GREEN_DIM: Color32 = Color32::from_rgb(6, 78, 59);     // #064e3b Deep Emerald
pub const COLOR_WARNING_ORANGE: Color32 = Color32::from_rgb(245, 158, 11);    // #f59e0b Amber
pub const COLOR_WARNING_ORANGE_DIM: Color32 = Color32::from_rgb(120, 53, 15);  // #78350f
pub const COLOR_DANGER_RED: Color32 = Color32::from_rgb(239, 68, 68);        // #ef4444 Ruby Red
pub const COLOR_DANGER_RED_DIM: Color32 = Color32::from_rgb(127, 29, 29);     // #7f1d1d

pub const COLOR_BG_CANVAS: Color32 = Color32::from_rgb(11, 15, 25);           // #0b0f19 Deep Space Canvas
pub const COLOR_BG_HEADER: Color32 = Color32::from_rgb(20, 27, 45);           // #141b2d Elevated Header Navbar
pub const COLOR_BG_STATS: Color32 = Color32::from_rgb(15, 22, 38);            // #0f1626 High-Tech Telemetry Bar
pub const COLOR_BG_CARD: Color32 = Color32::from_rgb(26, 36, 59);             // #1a243b Elevated Task Card
pub const COLOR_BG_CARD_HOVER: Color32 = Color32::from_rgb(38, 52, 84);       // #263454 Hover Card
pub const COLOR_BG_CARD_SELECTED: Color32 = Color32::from_rgb(30, 58, 138);    // #1e3a8a Selected Glowing Row
pub const COLOR_BG_INPUT: Color32 = Color32::from_rgb(15, 23, 42);            // #0f172a Input Background
pub const COLOR_TRACK_BG: Color32 = Color32::from_rgb(30, 41, 59);            // #1e293b Progress Bar Track

pub const COLOR_BORDER: Color32 = Color32::from_rgb(45, 59, 87);              // #2d3b57 Polished Border
pub const COLOR_BORDER_STRONG: Color32 = Color32::from_rgb(71, 85, 105);       // #475569 Strong Border
pub const COLOR_BORDER_FOCUS: Color32 = Color32::from_rgb(59, 130, 246);       // #3b82f6 Focus Glow

pub const COLOR_TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);     // #f8fafc Crisp Pure White
pub const COLOR_TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);   // #94a3b8 Clean Subtext
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);       // #64748b Muted Labels

/// Configure egui global visuals to a stunning modern high-contrast dark theme
pub fn configure_visuals(ctx: &Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = COLOR_BG_CANVAS;
    visuals.window_fill = COLOR_BG_HEADER;
    visuals.extreme_bg_color = COLOR_BG_INPUT;
    visuals.faint_bg_color = COLOR_BG_STATS;

    visuals.window_rounding = Rounding::same(12.0);
    visuals.menu_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    visuals.window_shadow.blur = 24.0;
    visuals.window_shadow.spread = 4.0;
    visuals.window_shadow.color = Color32::from_black_alpha(140);

    // Non-interactive widgets
    visuals.widgets.noninteractive.bg_fill = COLOR_BG_CARD;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);
    visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

    // Inactive widgets (buttons, inputs)
    visuals.widgets.inactive.bg_fill = COLOR_BG_CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_PRIMARY);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);

    // Hovered widgets
    visuals.widgets.hovered.bg_fill = COLOR_BG_CARD_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_BORDER_STRONG);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);

    // Active (pressed) widgets
    visuals.widgets.active.bg_fill = COLOR_ACCENT_BLUE_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, COLOR_ACCENT_BLUE);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::same(8.0);

    // Open dropdowns / context menus
    visuals.widgets.open.bg_fill = COLOR_BG_HEADER;
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

/// Primary Action Button (Modern Vibrant Blue Gradient style)
pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .strong()
            .size(13.0)
            .color(Color32::WHITE),
    )
    .fill(COLOR_ACCENT_BLUE)
    .stroke(Stroke::NONE)
    .rounding(Rounding::same(8.0))
    .min_size(Vec2::new(0.0, 28.0));

    ui.add(button)
}

/// Secondary Outlined Button
pub fn secondary_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .size(12.5)
            .color(COLOR_TEXT_PRIMARY),
    )
    .fill(COLOR_BG_CARD)
    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
    .rounding(Rounding::same(8.0))
    .min_size(Vec2::new(0.0, 28.0));

    ui.add(button)
}

/// Danger Action Button (Ruby Red style)
pub fn danger_button(ui: &mut Ui, text: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .size(12.5)
            .color(Color32::WHITE),
    )
    .fill(COLOR_DANGER_RED)
    .stroke(Stroke::NONE)
    .rounding(Rounding::same(8.0))
    .min_size(Vec2::new(0.0, 28.0));

    ui.add(button)
}

/// Minimal Icon / Toolbar Button
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    let button = egui::Button::new(
        RichText::new(icon)
            .size(13.5)
            .color(COLOR_TEXT_PRIMARY),
    )
    .fill(COLOR_BG_CARD)
    .stroke(Stroke::new(1.0_f32, COLOR_BORDER))
    .rounding(Rounding::same(8.0))
    .min_size(Vec2::new(30.0, 28.0));

    ui.add(button).on_hover_text(tooltip)
}

/// Renders a rounded pill badge (e.g., `NATIVE`, `16x Range`, `⚡ 18ms`)
pub fn pill_badge(ui: &mut Ui, text: &str, bg: Color32, fg: Color32) {
    let frame = egui::Frame::none()
        .fill(bg)
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::symmetric(8.0, 3.0));

    frame.show(ui, |ui| {
        ui.label(
            RichText::new(text)
                .size(11.0)
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
