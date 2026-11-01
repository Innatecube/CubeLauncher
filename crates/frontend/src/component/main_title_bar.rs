use bridge::handle::BackendHandle;
use gpui::prelude::*;
use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    plot::shape::{Arc, ArcData},
    Sizable,
};
use schema::pandora_update::UpdatePrompt;

use crate::{
    component::{generic_title_bar::TitleBar, page_path::PagePath},
    entity::task_manager::TaskManager,
    icon::PandoraIcon,
};

#[derive(IntoElement)]
pub struct MainTitleBar {
    pub page_path: PagePath,
    pub controls: AnyElement,
    pub update: Option<UpdatePrompt>,
    pub send: BackendHandle,
    pub tasks: Entity<TaskManager>,
}

struct RingPrepaintState {
    pct: f32,
    actual_inner_radius: f32,
    actual_outer_radius: f32,
    bounds: Bounds<Pixels>,
}

fn render_progress_ring(pct: f32) -> impl IntoElement {
    canvas(
        move |bounds: Bounds<Pixels>, _window: &mut Window, _cx: &mut App| {
            let stroke_width = 2.5_f32;
            let actual_size = bounds.size.width.min(bounds.size.height);
            let actual_radius = (actual_size.as_f32() - stroke_width) / 2.0;
            RingPrepaintState {
                pct,
                actual_inner_radius: (actual_radius - stroke_width / 2.0).max(0.0),
                actual_outer_radius: actual_radius + stroke_width / 2.0,
                bounds,
            }
        },
        move |_bounds, prepaint, window: &mut Window, _cx: &mut App| {
            let arc = Arc::new()
                .inner_radius(prepaint.actual_inner_radius)
                .outer_radius(prepaint.actual_outer_radius);

            // 1. Subtle full background track ring (360 degrees)
            arc.paint(
                &ArcData {
                    data: &(),
                    index: 0,
                    value: 100.0,
                    start_angle: 0.0,
                    end_angle: std::f32::consts::TAU,
                    pad_angle: 0.0,
                },
                rgb(0x00e5ff).opacity(0.2),
                None,
                None,
                &prepaint.bounds,
                window,
            );

            // 2. Active cyan progress arc from 12 o'clock (0.0) clockwise by (pct * TAU)
            if prepaint.pct > 0.005 {
                let end_angle = prepaint.pct * std::f32::consts::TAU;
                arc.paint(
                    &ArcData {
                        data: &(),
                        index: 1,
                        value: prepaint.pct * 100.0,
                        start_angle: 0.0,
                        end_angle,
                        pad_angle: 0.0,
                    },
                    rgb(0x00e5ff),
                    None,
                    None,
                    &prepaint.bounds,
                    window,
                );
            }
        },
    )
    .absolute()
    .size_full()
}

impl RenderOnce for MainTitleBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let snapshots = self.tasks.read(cx).snapshots();
        // Only show modpack/instance installs in the download icon
        let active_snapshots: Vec<_> = snapshots.iter().filter(|s| !s.is_finished && s.is_modpack).cloned().collect();
        let active_count = active_snapshots.len();

        let mut right_items = Vec::new();

        let toggle_tasks = self.tasks.clone();

        // Download icon button
        let download_icon = if active_count > 0 {
            // IMAGE 2: Active state — download icon with a circular progress indicator
            let primary = active_snapshots.first().unwrap();
            let pct = if primary.total > 0 {
                (primary.count as f32 / primary.total as f32).clamp(0.01, 1.0)
            } else if primary.count > 0 {
                0.2
            } else {
                0.05
            };

            div()
                .id("tasks-active-btn")
                .occlude()
                .size(px(28.0))
                .rounded_none()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .relative()
                .hover(|s| s.bg(rgba(0x00e5ff18)))
                .on_mouse_down(MouseButton::Left, |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .on_click(move |_, _, cx| {
                    toggle_tasks.update(cx, |t, cx| t.toggle_flyout(cx));
                })
                // Circular progress ring showing exact download percentage
                .child(render_progress_ring(pct))
                // Centered cyan download arrow icon
                .child(
                    div()
                        .size(px(14.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(0x00e5ff))
                        .child(PandoraIcon::Download)
                )
                .into_any_element()
        } else {
            // IMAGE 1: Idle state — simple muted download icon, no ring
            div()
                .id("tasks-idle-btn")
                .occlude()
                .size(px(28.0))
                .rounded_none()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_color(rgb(0x94a3b8))
                .hover(|s| s.text_color(rgb(0xe2e8f0)).bg(rgb(0x232733)))
                .on_mouse_down(MouseButton::Left, |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .on_click(move |_, _, cx| {
                    toggle_tasks.update(cx, |t, cx| t.toggle_flyout(cx));
                })
                .child(
                    div()
                        .size(px(16.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(PandoraIcon::Download)
                )
                .into_any_element()
        };

        right_items.push(download_icon);

        // 2. Existing Update Button (if update available)
        if let Some(update) = self.update {
            right_items.push(
                Button::new("update")
                    .label(t::system::update::available())
                    .success()
                    .compact()
                    .small()
                    .ml_2()
                    .icon(PandoraIcon::Download)
                    .on_click({
                        let send = self.send.clone();
                        move |_, window, cx| {
                            crate::modals::update_prompt::open_update_prompt(update.clone(), send.clone(), window, cx);
                        }
                    })
                    .into_any_element()
            );
        }

        TitleBar {
            left_content: vec![
                div().overflow_hidden().pr_8().child(self.page_path).into_any_element(),
                self.controls,
            ],
            right_content: right_items,
            content_only: !crate::root::should_render_custom_titlebar(),
        }
    }
}
