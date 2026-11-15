use eframe::egui::{self, Color32, Pos2, Shape, Stroke, Vec2};

pub fn mic_icon(painter: &egui::Painter, center: Pos2, size: f32, color: Color32, gap: Color32, slash_progress: f32) {
    let s = size / 24.0;
    let p = |x: f32, y: f32| center + Vec2::new((x - 12.0) * s, (y - 12.0) * s);
    let stroke = Stroke::new(1.9 * s, color);

    let capsule = egui::Rect::from_min_max(p(9.0, 3.0), p(15.0, 14.0));
    painter.rect_filled(capsule, egui::CornerRadius::same((3.0 * s) as u8), color);

    let arc_points: Vec<Pos2> = (0..=24)
        .map(|i| {
            let angle = std::f32::consts::PI * (i as f32 / 24.0);
            p(12.0 - 7.0 * angle.cos(), 11.0 + 6.0 * angle.sin())
        })
        .collect();
    painter.add(Shape::line(arc_points, stroke));

    painter.line_segment([p(12.0, 17.0), p(12.0, 21.0)], stroke);
    painter.line_segment([p(8.5, 21.0), p(15.5, 21.0)], stroke);

    if slash_progress > 0.0 {
        let start = p(3.0, 3.0);
        let end = p(21.0, 21.0);
        let tip = start + (end - start) * slash_progress.clamp(0.0, 1.0);
        painter.line_segment([start, tip], Stroke::new(4.4 * s, gap));
        painter.line_segment([start, tip], Stroke::new(2.0 * s, color));
    }
}
