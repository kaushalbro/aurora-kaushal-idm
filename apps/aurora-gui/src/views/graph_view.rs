use crate::theme::{self, COLOR_ACCENT_CYAN};
use egui::{Color32, Pos2, Rounding, Stroke, Ui, Vec2};

pub fn render_speed_graph(ui: &mut Ui, speed_history: &[f64], current_speed: f64, peak_speed: f64) {
    let desired_size = Vec2::new(ui.available_width(), 60.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

    let painter = ui.painter_at(rect);

    // Background Canvas
    painter.rect_filled(rect, Rounding::same(6.0), theme::COLOR_BG_INPUT);
    painter.rect_stroke(rect, Rounding::same(6.0), Stroke::new(1.0_f32, theme::COLOR_BORDER));

    if speed_history.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Real-Time Throughput Telemetry",
            egui::FontId::proportional(11.0),
            theme::COLOR_TEXT_MUTED,
        );
        return;
    }

    let max_val = peak_speed.max(current_speed).max(1024.0 * 1024.0); // Minimum 1 MB/s scale
    let num_points = speed_history.len();
    if num_points < 2 {
        return;
    }

    let pad_x = 8.0_f32;
    let pad_y = 6.0_f32;
    let w = rect.width() - 2.0 * pad_x;
    let h = rect.height() - 2.0 * pad_y;

    // Build polyline points
    let points: Vec<Pos2> = speed_history
        .iter()
        .enumerate()
        .map(|(i, &speed)| {
            let x = rect.left() + pad_x + (i as f32 / (num_points - 1) as f32) * w;
            let ratio = (speed as f32 / max_val as f32).clamp(0.0, 1.0);
            let y = rect.bottom() - pad_y - (ratio * h);
            Pos2::new(x, y)
        })
        .collect();

    // Draw area fill
    let mut fill_mesh = egui::Mesh::default();
    let fill_color = Color32::from_rgba_premultiplied(0, 122, 255, 35);
    for i in 0..points.len() - 1 {
        let p1 = points[i];
        let p2 = points[i + 1];
        let b1 = Pos2::new(p1.x, rect.bottom() - pad_y);
        let b2 = Pos2::new(p2.x, rect.bottom() - pad_y);

        fill_mesh.add_triangle(
            fill_mesh.vertices.len() as u32,
            fill_mesh.vertices.len() as u32 + 1,
            fill_mesh.vertices.len() as u32 + 2,
        );
        fill_mesh.colored_vertex(p1, fill_color);
        fill_mesh.colored_vertex(p2, fill_color);
        fill_mesh.colored_vertex(b2, fill_color);

        fill_mesh.add_triangle(
            fill_mesh.vertices.len() as u32,
            fill_mesh.vertices.len() as u32 + 1,
            fill_mesh.vertices.len() as u32 + 2,
        );
        fill_mesh.colored_vertex(p1, fill_color);
        fill_mesh.colored_vertex(b2, fill_color);
        fill_mesh.colored_vertex(b1, fill_color);
    }
    painter.add(fill_mesh);

    // Draw smooth line
    for i in 0..points.len() - 1 {
        painter.line_segment([points[i], points[i + 1]], Stroke::new(1.5_f32, COLOR_ACCENT_CYAN));
    }

    // Peak speed indicator text
    let peak_str = format!("Peak: {:.2} MB/s", peak_speed / (1024.0 * 1024.0));
    painter.text(
        Pos2::new(rect.left() + pad_x + 4.0, rect.top() + pad_y + 2.0),
        egui::Align2::LEFT_TOP,
        peak_str,
        egui::FontId::proportional(10.0),
        theme::COLOR_TEXT_SECONDARY,
    );

    // Current speed text
    let cur_str = format!("{:.2} MB/s", current_speed / (1024.0 * 1024.0));
    painter.text(
        Pos2::new(rect.right() - pad_x - 4.0, rect.top() + pad_y + 2.0),
        egui::Align2::RIGHT_TOP,
        cur_str,
        egui::FontId::proportional(11.0),
        COLOR_ACCENT_CYAN,
    );
}
