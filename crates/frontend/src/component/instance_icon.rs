use gpui::{prelude::*, *};
use gpui_component::Icon;
use schema::unique_bytes::UniqueBytes;

use crate::png_render_cache::{self, ImageTransformation};

/// Deterministically picks a vibrant, high-saturation color from Modrinth's palette.
pub fn get_vibrant_icon_color(seed: &str) -> Hsla {
    let lower = seed.to_lowercase();
    if lower.contains("grass") || lower.contains("earth") || lower.contains("water") || lower.contains("sky") {
        return rgb(0x0284C7).into(); // Azure / Sky Blue #0284C7
    }
    if lower.contains("wand") || lower.contains("rod") || lower.contains("magic") || lower.contains("potion") {
        return rgb(0x9333EA).into(); // Royal Purple #9333EA
    }
    if lower.contains("dragon") || lower.contains("ender") || lower.contains("void") || lower.contains("sword") {
        return rgb(0x4F46E5).into(); // Electric Indigo #4F46E5
    }
    if lower.contains("command") || lower.contains("fuchsia") || lower.contains("heart") {
        return rgb(0xDB2777).into(); // Hot Fuchsia #DB2777
    }
    if lower.contains("emerald") || lower.contains("slime") || lower.contains("tree") || lower.contains("leaf") {
        return rgb(0x10B981).into(); // Emerald Green #10B981
    }
    if lower.contains("backpack") || lower.contains("fire") || lower.contains("lava") || lower.contains("chest") {
        return rgb(0xEA580C).into(); // Sunset Orange #EA580C
    }
    if lower.contains("redstone") || lower.contains("tnt") || lower.contains("ruby") {
        return rgb(0xE11D48).into(); // Ruby Rose #E11D48
    }

    const PALETTE: [u32; 8] = [
        0x0284C7, // Azure
        0x9333EA, // Royal Purple
        0x4F46E5, // Electric Indigo
        0xDB2777, // Vibrant Fuchsia
        0x10B981, // Emerald Green
        0xEA580C, // Sunset Orange
        0xE11D48, // Ruby Rose
        0x0D9488, // Cyan Teal
    ];

    let mut hash: u32 = 5381;
    for b in seed.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    let hex = PALETTE[(hash as usize) % PALETTE.len()];
    rgb(hex).into()
}

/// Renders a prominent Modrinth-style squircle instance icon tile with vibrant background.
pub fn render_instance_tile(
    icon: Option<UniqueBytes>,
    name: &str,
    fallback_icon: Option<&str>,
    size_px: f32,
    _radius_px: f32,
    cx: &mut App,
) -> AnyElement {
    let transform_size = (size_px * 2.0).round().max(64.0) as u32;

    let (icon_element, bg_color) = if let Some(bytes) = icon {
        let transform = ImageTransformation::Resize {
            width: transform_size,
            height: transform_size,
        };
        (
            div()
                .size(px(size_px))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    png_render_cache::render_with_transform(bytes, transform, cx)
                        .size(px(size_px))
                        .rounded_none()
                )
                .into_any_element(),
            rgba(0x1a1e2680).into(),
        )
    } else {
        let icon_path = fallback_icon.unwrap_or("icons/box.svg");
        let icon_size = size_px * 0.6;
        (
            Icon::default()
                .path(icon_path)
                .size(px(icon_size))
                .into_any_element(),
            get_vibrant_icon_color(name),
        )
    };

    div()
        .size(px(size_px))
        .min_size(px(size_px))
        .max_size(px(size_px))
        .rounded_none()
        .bg(bg_color)
        .flex()
        .items_center()
        .justify_center()
        .overflow_hidden()
        .flex_none()
        .child(icon_element)
        .into_any_element()
}

/// Renders a compact 34x34 squircle icon tile for the sidebar (recent instances or navigation).
pub fn render_sidebar_tile(
    icon: Option<UniqueBytes>,
    name: &str,
    fallback_icon: Option<&str>,
    cx: &mut App,
) -> AnyElement {
    render_instance_tile(icon, name, fallback_icon, 34.0, 8.0, cx)
}

fn render_squircle_progress_ring(pct: f32, stroke_width: f32, radius: f32) -> impl IntoElement {
    canvas(
        move |bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut App| {
            bounds
        },
        move |bounds, _prepaint, window: &mut Window, _cx: &mut App| {
            let inset = stroke_width / 2.0;
            let x = bounds.origin.x + px(inset);
            let y = bounds.origin.y + px(inset);
            let w = (bounds.size.width - px(stroke_width)).max(px(0.0));
            let h = (bounds.size.height - px(stroke_width)).max(px(0.0));
            let r = (radius - inset).min(w.as_f32() / 2.0).min(h.as_f32() / 2.0).max(0.0);

            let straight_w = (w - px(r * 2.0)).max(px(0.0));
            let straight_h = (h - px(r * 2.0)).max(px(0.0));
            let p_len = (straight_w * 2.0 + straight_h * 2.0 + px(std::f32::consts::TAU * r)).as_f32();

            let build_squircle = || {
                let mut pb = PathBuilder::stroke(px(stroke_width));
                pb.move_to(point(x + w / 2.0, y));
                pb.line_to(point(x + w - px(r), y));
                pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(x + w, y + px(r)));
                pb.line_to(point(x + w, y + h - px(r)));
                pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(x + w - px(r), y + h));
                pb.line_to(point(x + px(r), y + h));
                pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(x, y + h - px(r)));
                pb.line_to(point(x, y + px(r)));
                pb.arc_to(point(px(r), px(r)), px(0.0), false, true, point(x + px(r), y));
                pb.line_to(point(x + w / 2.0, y));
                pb.close();
                pb
            };

            // 1. Subtle full background track
            let bg_pb = build_squircle();
            if let Ok(path) = bg_pb.build() {
                window.paint_path(path, rgb(0x00e5ff).opacity(0.18));
            }

            // 2. Active cyan progress arc along the squircle perimeter
            if pct > 0.005 {
                let active_len = p_len * pct.clamp(0.0, 1.0);
                let gap_len = p_len * 2.0;
                let fg_pb = build_squircle().dash_array(&[px(active_len), px(gap_len)]);
                if let Ok(path) = fg_pb.build() {
                    window.paint_path(path, rgb(0x00e5ff));
                }
            }
        },
    )
    .absolute()
    .size_full()
}

/// Renders an instance icon tile with an optional glowing cyan downloading progress squircle ring.
pub fn render_instance_tile_with_download(
    icon: Option<UniqueBytes>,
    name: &str,
    fallback_icon: Option<&str>,
    size_px: f32,
    radius_px: f32,
    download_progress: Option<f32>,
    cx: &mut App,
) -> AnyElement {
    let tile = render_instance_tile(icon, name, fallback_icon, size_px, radius_px, cx);
    if let Some(pct) = download_progress {
        div()
            .relative()
            .size(px(size_px))
            .min_size(px(size_px))
            .max_size(px(size_px))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .size_full()
                    .opacity(0.35)
                    .child(tile)
            )
            .child(
                div()
                    .absolute()
                    .size_full()
                    .bg(rgba(0x000000a0))
                    .rounded_none()
            )
            .child(render_squircle_progress_ring(pct, 2.5, radius_px))
            .into_any_element()
    } else {
        tile
    }
}

/// Renders a sidebar tile with an optional glowing cyan downloading progress squircle ring.
pub fn render_sidebar_tile_with_download(
    icon: Option<UniqueBytes>,
    name: &str,
    fallback_icon: Option<&str>,
    download_progress: Option<f32>,
    cx: &mut App,
) -> AnyElement {
    let tile = render_instance_tile(icon, name, fallback_icon, 34.0, 8.0, cx);
    if let Some(pct) = download_progress {
        div()
            .relative()
            .size(px(34.0))
            .min_size(px(34.0))
            .max_size(px(34.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .size_full()
                    .opacity(0.35)
                    .child(tile)
            )
            .child(
                div()
                    .absolute()
                    .size_full()
                    .bg(rgba(0x000000a0))
                    .rounded_none()
            )
            .child(render_squircle_progress_ring(pct, 2.0, 8.0))
            .into_any_element()
    } else {
        tile
    }
}
