use aurora_core::types::{Segment, SegmentState};
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Renders a responsive, color-coded visualizer of file segments and their byte completion states.
pub fn render_segment_visualization(
    ui: &mut Ui,
    segments: &[Segment],
    total_bytes: Option<u64>,
    height: f32,
) -> Response {
    let desired_size = Vec2::new(ui.available_width(), height);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let painter = ui.painter_at(rect);

    // Background track (macOS light track #e5e5ea)
    painter.rect_filled(rect, 4.0, Color32::from_rgb(229, 229, 234));
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0_f32, Color32::from_rgb(209, 209, 214)),
    );

    let total = match total_bytes {
        Some(t) if t > 0 => t as f64,
        _ => return response,
    };

    let width = rect.width();

    for seg in segments {
        let seg_start_x = rect.left() + (seg.range.start as f64 / total * width as f64) as f32;
        let seg_end_x = rect.left() + (seg.range.end as f64 / total * width as f64) as f32;
        let seg_width = (seg_end_x - seg_start_x).max(1.0);

        let dl_fraction = seg.progress_fraction() as f32;
        let completed_width = seg_width * dl_fraction;

        let seg_rect = Rect::from_min_size(
            Pos2::new(seg_start_x, rect.top() + 2.0),
            Vec2::new(seg_width, height - 4.0),
        );

        // Segment base color based on state
        let base_color = match seg.state {
            SegmentState::Completed => Color32::from_rgb(52, 199, 89),   // #34c759 Green
            SegmentState::Downloading => Color32::from_rgb(0, 122, 255), // #007aff Blue
            SegmentState::Failed => Color32::from_rgb(255, 59, 48),      // #ff3b30 Red
            SegmentState::Paused => Color32::from_rgb(255, 149, 0),      // #ff9500 Orange
            SegmentState::Pending => Color32::from_rgb(209, 209, 214),   // #d1d1d6 Light Gray
        };

        // Draw un-downloaded portion of the segment
        painter.rect_filled(seg_rect, 2.0, base_color.linear_multiply(0.35));

        // Draw downloaded completed portion
        if completed_width > 0.0 {
            let dl_rect = Rect::from_min_size(
                Pos2::new(seg_start_x, rect.top() + 2.0),
                Vec2::new(completed_width, height - 4.0),
            );
            painter.rect_filled(dl_rect, 2.0, Color32::from_rgb(52, 199, 89));
        }

        // Draw border separator between segments
        painter.line_segment(
            [
                Pos2::new(seg_start_x, rect.top() + 2.0),
                Pos2::new(seg_start_x, rect.bottom() - 2.0),
            ],
            Stroke::new(1.0_f32, Color32::WHITE),
        );
    }

    response
}
