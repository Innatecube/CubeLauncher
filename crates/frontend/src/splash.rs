use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
    },
    time::Instant,
};

use gpui::{
    prelude::*,
    div, px, rgb, rgba, size, Bounds, Context, ImageSource, IntoElement, ParentElement,
    Point, Render, Resource, Styled, TitlebarOptions, Window, WindowBounds, WindowControlArea,
    WindowDecorations, WindowOptions, App, AnyWindowHandle, InteractiveElement,
};
use gpui_component::{h_flex, v_flex, Root, StyledExt};

use crate::component::generic_title_bar::WindowControl;

pub struct SplashScreen {
    start_time: Instant,
    _is_alive: Arc<AtomicBool>,
}

impl SplashScreen {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>, is_alive: Arc<AtomicBool>) -> Self {
        Self {
            start_time: Instant::now(),
            _is_alive: is_alive,
        }
    }
}

impl Render for SplashScreen {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let duration = 1.35; // Total duration in seconds
        let t = (elapsed / duration).clamp(0.0, 1.0);

        // Continuous buttery-smooth cubic ease-out curve
        let progress = 1.0 - (1.0 - t).powi(3);

        if t < 1.0 {
            window.request_animation_frame();
        }

        let bar_width = 340.0;
        let fill_width = (bar_width * progress).max(4.0);

        // Top custom titlebar matching clean Windows style (seamless like Modrinth)
        let titlebar = h_flex()
            .w_full()
            .h(px(34.0))
            .bg(rgba(0x00000000))
            .items_center()
            .pl(px(12.0))
            .pr(px(0.0))
            // Left icon & branding
            .child(
                h_flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        gpui::img(ImageSource::Resource(Resource::Embedded("images/logo.png".into())))
                            .size(px(16.0))
                    )
                    .child(
                        div()
                            .font_family(crate::interface_config::DEFAULT_FONT)
                            .text_size(px(13.0))
                            .text_color(rgb(0xd4d4d4))
                            .child("CubeLauncher")
                    )
            )
            // Center draggable area
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .window_control_area(WindowControlArea::Drag)
            )
            // Right window controls (Minimize, Maximize/Restore, Close)
            .child(
                h_flex()
                    .items_center()
                    .gap_0()
                    .child(WindowControl::Minimize)
                    .child(if window.is_maximized() {
                        WindowControl::Restore
                    } else {
                        WindowControl::Maximize
                    })
                    .child(WindowControl::Close)
            );

        // Centered branding:
        // [CL Logo] cube launcher
        // -----------------------
        let center_content = v_flex()
            .items_center()
            .justify_center()
            .relative()
            .child(
                h_flex()
                    .items_center()
                    .gap(px(12.0))
                    // CubeLauncher logo mark
                    .child(
                        gpui::img(ImageSource::Resource(Resource::Embedded("images/logo.png".into())))
                            .size(px(38.0))
                    )
                    // "cube" in white + "launcher" in vibrant cyan with NoRisk font
                    .child(
                        h_flex()
                            .items_baseline()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .font_family(crate::interface_config::DEFAULT_FONT)
                                    .text_size(px(32.0))
                                    .font_bold()
                                    .text_color(rgb(0xffffff))
                                    .child("cube")
                            )
                            .child(
                                div()
                                    .font_family(crate::interface_config::DEFAULT_FONT)
                                    .text_size(px(32.0))
                                    .font_bold()
                                    .text_color(rgb(0x00f0ff))
                                    .child("launcher")
                            )
                    )
            )
            // Sleek progress bar - sharp corners, smooth animation
            .child(
                div()
                    .mt(px(20.0))
                    .w(px(bar_width))
                    .h(px(5.0))
                    .rounded_none()
                    .bg(rgb(0x23272a))
                    .relative()
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .h_full()
                            .w(px(fill_width))
                            .rounded_none()
                            .bg(rgb(0x00f0ff))
                    )
            );

        div()
            .size_full()
            .bg(rgb(0x0d1417))
            .overflow_hidden()
            .relative()
            .flex()
            .flex_col()
            // Top Titlebar
            .child(titlebar)
            // Main body area (Clean uniform dark background matching Modrinth, NO glitchy rectangles)
            .child(
                div()
                    .flex_1()
                    .relative()
                    .w_full()
                    .overflow_hidden()
                    // Centered branding & smooth progress bar
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(center_content)
                    )
            )
    }
}

pub fn open_splash_window(cx: &mut App) -> (AnyWindowHandle, Arc<AtomicBool>) {
    let is_alive = Arc::new(AtomicBool::new(true));
    let is_alive_clone = is_alive.clone();

    let options = WindowOptions {
        app_id: Some("CubeLauncher".into()),
        window_bounds: Some(WindowBounds::Maximized(Bounds::new(
            Point::default(),
            size(px(1920.0), px(1080.0)),
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("CubeLauncher".into()),
            appears_transparent: true,
            ..Default::default()
        }),
        app_owns_titlebar_drag: true,
        window_decorations: Some(WindowDecorations::Client),
        is_resizable: true,
        is_minimizable: true,
        ..Default::default()
    };

    let handle = cx.open_window(options, move |window, cx| {
        let splash = cx.new(|cx| SplashScreen::new(window, cx, is_alive_clone));
        cx.new(|cx| Root::new(splash, window, cx))
    }).unwrap();

    (handle.into(), is_alive)
}
