use std::sync::{atomic::{AtomicU8, Ordering}, Arc};

use bridge::{handle::BackendHandle, instance::InstanceID};
use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants}, v_flex, WindowExt
};

pub fn open_delete_instance(
    instance: InstanceID,
    instance_name: SharedString,
    backend_handle: BackendHandle,
    window: &mut Window,
    cx: &mut App,
) {
    let stage = Arc::new(AtomicU8::new(0));

    let title: SharedString = t::instance::delete_dialog::title(&instance_name).into();
    let warning_message: SharedString = t::instance::delete_dialog::warning(&instance_name).into();

    window.open_dialog(cx, move |dialog, _, _| {
        let stage_val = stage.load(Ordering::Relaxed);

        let content = match stage_val {
            0 => {
                v_flex()
                    .child(Button::new("delete").label(t::instance::delete_dialog::label()).on_click({
                        let stage = stage.clone();
                        move |_, _, _| {
                            stage.store(1, Ordering::Relaxed);
                        }
                    }))
            }
            1 => {
                v_flex()
                    .gap_2()
                    .child(warning_message.clone())
                    .child(Button::new("confirm").label(t::instance::delete_dialog::check()).danger().on_click({
                        let backend_handle = backend_handle.clone();
                        move |_, window, cx| {
                            backend_handle.send(bridge::message::MessageToBackend::DeleteInstance {
                                id: instance
                            });
                            window.close_all_dialogs(cx);
                        }
                    }))
            }
            _ => {
                unreachable!()
            }
        };

        dialog
            .on_ok({
                let backend_handle = backend_handle.clone();
                move |_, _, _| {
                    if stage_val != 1 {
                        return false;
                    }
                    backend_handle.send(bridge::message::MessageToBackend::DeleteInstance {
                        id: instance
                    });
                    true
                }
            })
            .title(title.clone())
            .child(content)
    });
}
