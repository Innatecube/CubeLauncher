use gpui::{prelude::*, *};
use gpui_component::{ActiveTheme, Icon};
use schema::unique_bytes::UniqueBytes;

use crate::{
    entity::instance::InstanceEntry,
    png_render_cache::{self, ImageTransformation},
};

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
    radius_px: f32,
    cx: &mut App,
) -> AnyElement {
    let bg_color = get_vibrant_icon_color(name);
    let transform_size = (size_px * 1.5).round().max(64.0) as u32;

    let icon_element = if let Some(bytes) = icon {
        let transform = ImageTransformation::Resize {
            width: transform_size,
            height: transform_size,
        };
        // 3D icons get ~12% padding inside squircle so drop shadows blend naturally.
        let img_size = size_px * 0.82;
        div()
            .size(px(img_size))
            .flex()
            .items_center()
            .justify_center()
            .child(
                png_render_cache::render_with_transform(bytes, transform, cx)
                    .size(px(img_size))
                    .rounded(px(radius_px * 0.5))
            )
            .into_any_element()
    } else {
        let icon_path = fallback_icon.unwrap_or("icons/box.svg");
        let icon_size = size_px * 0.6;
        Icon::default()
            .path(icon_path)
            .size(px(icon_size))
            .into_any_element()
    };

    div()
        .size(px(size_px))
        .min_size(px(size_px))
        .max_size(px(size_px))
        .rounded(px(radius_px))
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
