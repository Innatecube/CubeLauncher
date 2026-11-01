use gpui::{prelude::*, *};

/// Returns a full-size absolute canvas that draws a vanishing-point perspective
/// grid overlay — a subtle teal/cyan background texture that evokes the NoRisk
/// launcher aesthetic.
///
/// Place this as the **first** child of a `.relative()` container:
/// ```rust
/// div()
///     .relative()
///     .size_full()
///     .child(perspective_grid())   // behind everything
///     .child(real_content)
/// ```
pub fn perspective_grid() -> impl IntoElement {
    canvas(
        // prepaint: capture the layout bounds for use in the paint closure
        |bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut App| bounds,
        // paint: draw the grid lines
        move |_elem_bounds, bounds, window: &mut Window, _cx: &mut App| {
            let w = bounds.size.width.as_f32();
            let h = bounds.size.height.as_f32();

            if w < 1.0 || h < 1.0 {
                return;
            }

            // Vanishing point: horizontal centre, 30% from the top
            let vp_x = w * 0.5;
            let vp_y = h * 0.30;

            let origin_x = bounds.origin.x.as_f32();
            let origin_y = bounds.origin.y.as_f32();

            let stroke_px = px(0.8);

            // ── Radial lines from the vanishing point ──────────────────────────
            // Shoot rays from the VP to evenly spaced points along the bottom edge.
            let radial_count = 16usize;

            for i in 0..=radial_count {
                let t = i as f32 / radial_count as f32; // 0 → 1
                let target_x = origin_x + t * w;
                let target_y = origin_y + h;

                let mut pb = PathBuilder::stroke(stroke_px);
                pb.move_to(point(px(origin_x + vp_x), px(origin_y + vp_y)));
                pb.line_to(point(px(target_x), px(target_y)));

                if let Ok(path) = pb.build() {
                    // Subtle radial lines: ~9% opacity cyan
                    window.paint_path(path, rgba(0x00f0ff17));
                }
            }

            // ── Horizontal lines with quadratic (perspective) spacing ──────────
            // Lines bunch up near the vanishing point and spread out at the bottom.
            let horiz_count = 10usize;

            for i in 1..=horiz_count {
                let t = i as f32 / horiz_count as f32;
                // Quadratic easing: small gaps near VP, big gaps at bottom
                let frac = t * t;
                let y = origin_y + vp_y + (h - vp_y) * frac;

                // Lines near the VP are more transparent; bottom lines are bolder
                let alpha = ((0x08u32 as f32 + 0x0cu32 as f32 * t) as u32).clamp(0, 0xff);
                let color_u32 = 0x00f0ff00_u32 | alpha;
                let line_color = rgba(color_u32);

                let mut pb = PathBuilder::stroke(stroke_px);
                pb.move_to(point(px(origin_x), px(y)));
                pb.line_to(point(px(origin_x + w), px(y)));

                if let Ok(path) = pb.build() {
                    window.paint_path(path, line_color);
                }
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}
