use std::sync::Arc;

use bridge::modal_action::{ModalAction, ProgressTrackerFinishType};
use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Disableable, WindowExt, button::{Button, ButtonVariant, ButtonVariants}, notification::Notification, v_flex
};

use crate::{component::{
    error_alert::ErrorAlert,
    progress_bar::{ProgressBar, ProgressBarColor},
}, icon::PandoraIcon};

pub fn show_notification(
    window: &mut Window,
    cx: &mut App,
    error_title: SharedString,
    modal_action: ModalAction,
) {
    show_notification_with_note(window, cx, error_title, modal_action, Notification::new());
}

pub fn show_notification_with_note(
    window: &mut Window,
    cx: &mut App,
    error_title: SharedString,
    modal_action: ModalAction,
    notification: Notification
) {
    let notify = modal_action.get_notify();
    let task = window.spawn(cx, async move |cx| {
        loop {
            notify.notified().await;
            let res = cx.update_window(cx.window_handle(), |_, window, _| {
                window.refresh();
            });
            if res.is_err() {
                break;
            }
        }
    });

    let notification = notification
        .autohide(false)
        .content(move |notification, window, cx| {
            _ = &task; // Keep refresh task alive

            if let Some(error) = modal_action.get_error_message() {
                let error_widget = ErrorAlert::new(error_title.clone(), error.clone().into());
                return error_widget.into_any_element();
            }

            if modal_action.refcnt() <= 1 || modal_action.get_finished_at().is_some() {
                notification.dismiss(window, cx);
            }

            let (mut progress_entries, needs_animation) = render_progress_trackers(&modal_action, 0.0);
            if needs_animation {
                window.request_animation_frame();
            }

            if let Some(visit_url) = modal_action.get_visit_url() {
                let message = SharedString::new(Arc::clone(&visit_url.message));
                let url = Arc::clone(&visit_url.url);
                progress_entries.push(div().p_3().child(Button::new("visit").info().icon(PandoraIcon::ExternalLink).label(message).on_click(
                    move |_, _, cx| {
                        cx.open_url(&url);
                    },
                )));
            }

            v_flex().gap_2().children(progress_entries).into_any_element()
        });
    window.push_notification(notification, cx);
}

pub fn watch_action_errors(
    window: &mut Window,
    cx: &mut App,
    error_title: SharedString,
    modal_action: ModalAction,
) {
    let notify = modal_action.get_notify();
    let modal_action_clone = modal_action.clone();
    let error_title = error_title.clone();
    window.spawn(cx, async move |cx| {
        loop {
            if let Some(error) = modal_action_clone.get_error_message() {
                let _ = cx.update_window(cx.window_handle(), move |_, window, cx| {
                    crate::root::capture_backdrop_blur(window, cx);
                    window.open_dialog(cx, move |dialog, _, _cx| {
                        let error_widget = ErrorAlert::new(error_title.clone(), error.clone().into());
                        let dismiss = Button::new("ok")
                            .primary()
                            .label(t::common::ok())
                            .on_click(|_, window, cx| window.close_dialog(cx));
                        dialog
                            .w(px(460.0))
                            .close_button(true)
                            .overlay_closable(true)
                            .child(v_flex().gap_3().pt_1().child(error_widget))
                            .footer(dismiss)
                    });
                });
                break;
            }
            if modal_action_clone.refcnt() <= 1 || modal_action_clone.get_finished_at().is_some() {
                break;
            }
            notify.notified().await;
        }
    }).detach();
}



fn render_progress_trackers(modal_action: &ModalAction, elapsed_modal: f32) -> (Vec<Div>, bool) {
    modal_action.write_trackers(|trackers| {
        let mut progress_entries = Vec::with_capacity(trackers.len());
        let mut needs_animation = false;

        let mut finishing_tracker_slots = 8;
        trackers.retain(|tracker| {
            if let Some(finished_at) = tracker.get_finished_at() {
                let finish_type = tracker.finish_type();
                if finish_type == ProgressTrackerFinishType::Fast {
                    return false;
                }

                let elapsed = (finished_at.elapsed().as_secs_f32() - elapsed_modal).max(0.0);
                if elapsed >= 2.0 {
                    return false;
                }
            } else {
                finishing_tracker_slots -= 1;
            }
            true
        });

        for tracker in &*trackers {
            let mut opacity = 1.0;

            let mut progress_bar = ProgressBar::new();
            if let Some(progress_amount) = tracker.get_float() {
                progress_bar.amount = progress_amount;
            }

            if let Some(finished_at) = tracker.get_finished_at() {
                if finishing_tracker_slots <= 0 {
                    continue;
                }
                finishing_tracker_slots -= 1;

                let elapsed = finished_at.elapsed().as_secs_f32();
                let elapsed_fade = (elapsed - elapsed_modal).max(0.0);
                if elapsed_fade >= 1.0 {
                    opacity = (2.0 - elapsed_fade).max(0.0);
                }

                let finish_type = tracker.finish_type();
                if finish_type == ProgressTrackerFinishType::Error {
                    progress_bar.color = ProgressBarColor::Error;
                } else {
                    progress_bar.color = ProgressBarColor::Success;
                }
                if elapsed <= 0.5 {
                    progress_bar.color_scale = elapsed * 2.0;
                }

                needs_animation = true;
            }

            let title = tracker.get_title();
            progress_entries.push(div().gap_3().child(SharedString::from(title)).child(progress_bar).opacity(opacity));
        }
        (progress_entries, needs_animation)
    })
}



pub fn show_modal(
    window: &mut Window,
    cx: &mut App,
    title: SharedString,
    error_title: SharedString,
    modal_action: ModalAction,
) {
    crate::root::capture_backdrop_blur(window, cx);

    let notify = modal_action.get_notify();
    let task = window.spawn(cx, async move |cx| {
        loop {
            notify.notified().await;
            let res = cx.update_window(cx.window_handle(), |_, window, _| {
                window.refresh();
            });
            if res.is_err() {
                break;
            }
        }
    });

    let modal_action_clone = modal_action.clone();
    window.open_dialog(cx, move |dialog, window, cx| {
        _ = &task; // Keep task alive for refreshing

        let (content, footer) = if let Some(error) = modal_action_clone.get_error_message() {
            let error_widget = ErrorAlert::new(error_title.clone(), error.clone().into());
            let dismiss = Button::new("ok")
                .primary()
                .label(t::common::ok())
                .on_click(|_, window, cx| window.close_dialog(cx));

            (
                error_widget.into_any_element(),
                dismiss.into_any_element(),
            )
        } else {
            if modal_action_clone.refcnt() <= 1 {
                modal_action_clone.set_finished();
            }

            let visit_url = modal_action_clone.get_visit_url();

            let mut is_finishing = false;
            let mut elapsed_modal = 0.0;
            if let Some(finished_at) = modal_action_clone.get_finished_at() {
                is_finishing = true;

                let prevent_finish = visit_url.as_ref().map(|v| v.prevent_auto_finish).unwrap_or(false);

                if !prevent_finish {
                    elapsed_modal = finished_at.elapsed().as_secs_f32();
                    window.request_animation_frame();
                    if elapsed_modal >= 2.0 {
                        window.close_dialog(cx);
                    }
                }
            }

            let (mut progress_entries, needs_animation) = render_progress_trackers(&modal_action_clone, elapsed_modal);

            if needs_animation {
                window.request_animation_frame();
            }

            if let Some(visit_url) = visit_url {
                let url = Arc::clone(&visit_url.url);

                // Auto-open in external browser on initial appearance
                let auto_opened = window.use_keyed_state("modal-auto-open-browser", cx, |_, _| false);
                if !*auto_opened.read(cx) {
                    cx.open_url(&url);
                    auto_opened.update(cx, |opened, _| *opened = true);
                }

                let url_for_open = url.clone();
                let url_for_copy = url.clone();

                progress_entries.push(
                    v_flex()
                        .gap_3()
                        .p_3()
                        .bg(cx.theme().secondary.opacity(0.35))
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child("A browser window has opened for you to authenticate with Microsoft. If it didn't open automatically, use the buttons below:")
                        )
                        .child(
                            gpui_component::h_flex()
                                .gap_2()
                                .child(
                                    Button::new("visit")
                                        .info()
                                        .icon(PandoraIcon::ExternalLink)
                                        .label("Open in Browser")
                                        .on_click(move |_, _, cx| {
                                            cx.open_url(&url_for_open);
                                        })
                                )
                                .child(
                                    Button::new("copy-url")
                                        .secondary()
                                        .icon(PandoraIcon::Copy)
                                        .label("Copy Link")
                                        .on_click(move |_, _, cx| {
                                            cx.write_to_clipboard(ClipboardItem::new_string(url_for_copy.to_string()));
                                        })
                                )
                        )
                );
            }

            let progress = v_flex().gap_2().children(progress_entries);

            if is_finishing {
                if modal_action_clone.has_requested_cancel() {
                    window.close_dialog(cx);
                }
                let dismiss = Button::new("ok")
                    .with_variant(ButtonVariant::Secondary)
                    .label(t::common::ok())
                    .on_click(|_, window, cx| window.close_dialog(cx));
                (progress.into_any_element(), dismiss.into_any_element())
            } else {
                let cancel = modal_action_clone.request_cancel.clone();
                let cancel_btn = Button::new("cancel")
                    .disabled(modal_action_clone.has_requested_cancel())
                    .label(t::common::cancel())
                    .on_click(move |_, _, _| cancel.cancel());
                (progress.into_any_element(), cancel_btn.into_any_element())
            }
        };

        dialog
            .w(px(460.0))
            .close_button(true)
            .overlay_closable(true)
            .title(title.clone())
            .child(v_flex().gap_3().pt_1().child(content))
            .footer(footer)
    });
}
