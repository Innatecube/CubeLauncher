use std::rc::Rc;

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Disableable, StyledExt,
    h_flex, v_flex,
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState},
    switch::Switch,
};
use schema::instance::InstanceJvmBinaryConfiguration;

use crate::{
    component::path_label::PathLabel,
    settings::{SettingGroup, SettingItem, SettingItemWidget, SettingPage},
};

pub fn get_system_ram_mb() -> u32 {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        if unsafe { GlobalMemoryStatusEx(&mut status) }.is_ok() {
            let mb = (status.ullTotalPhys / (1024 * 1024)) as u32;
            if mb > 0 {
                return mb.max(4096);
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    if let Some(kb) = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()) {
                        return ((kb / 1024) as u32).max(4096);
                    }
                }
            }
        }
    }
    16384
}

pub(super) fn create_page() -> SettingPage {
    SettingPage {
        title: t::settings::java,
        groups: vec![
            SettingGroup {
                title: Some(t::settings::java::defaults),
                items: vec![
                    SettingItem {
                        title: t::settings::java::defaults::enable_memory,
                        description: t::settings::java::defaults::enable_memory_desc,
                        widget: SettingItemWidget::Backend(Rc::new(|backend, _, cx| {
                            let enabled = backend.memory.as_ref().is_some_and(|memory| memory.enabled);
                            Switch::new("enable-default-memory").checked(enabled).on_click(cx.listener(|root, val, _, cx| {
                                let Some(backend) = root.backend_config().cloned() else {
                                    return;
                                };
                                let mut memory = backend.memory.unwrap_or_default();
                                if memory.enabled == *val {
                                    return;
                                }
                                memory.enabled = *val;
                                root.set_launch_defaults(
                                    Some(memory),
                                    backend.jvm_flags.clone(),
                                    backend.jvm_binary.clone(),
                                    cx,
                                );
                            })).into_any_element()
                        })),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::java::defaults::memory_max,
                        description: t::settings::java::defaults::memory_max_desc,
                        widget: create_global_memory_slider_widget(),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::java::defaults::enable_jvm_flags,
                        description: t::settings::java::defaults::enable_jvm_flags_desc,
                        widget: SettingItemWidget::Backend(Rc::new(|backend, _, cx| {
                            let enabled = backend.jvm_flags.as_ref().is_some_and(|flags| flags.enabled);
                            Switch::new("enable-default-jvm-flags").checked(enabled).on_click(cx.listener(|root, val, _, cx| {
                                let Some(backend) = root.backend_config().cloned() else {
                                    return;
                                };
                                let mut jvm_flags = backend.jvm_flags.unwrap_or_default();
                                if jvm_flags.enabled == *val {
                                    return;
                                }
                                jvm_flags.enabled = *val;
                                root.set_launch_defaults(
                                    backend.memory.clone(),
                                    Some(jvm_flags),
                                    backend.jvm_binary.clone(),
                                    cx,
                                );
                            })).into_any_element()
                        })),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::java::defaults::jvm_flags,
                        description: t::settings::java::defaults::jvm_flags_desc,
                        widget: create_jvm_flags_widget(),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::java::defaults::enable_jvm_binary,
                        description: t::settings::java::defaults::enable_jvm_binary_desc,
                        widget: SettingItemWidget::Backend(Rc::new(|backend, _, cx| {
                            let enabled = backend.jvm_binary.as_ref().is_some_and(|binary| binary.enabled);
                            Switch::new("enable-default-jvm-binary").checked(enabled).on_click(cx.listener(|root, val, _, cx| {
                                let Some(backend) = root.backend_config().cloned() else {
                                    return;
                                };
                                let mut jvm_binary = backend.jvm_binary.unwrap_or_default();
                                if jvm_binary.enabled == *val {
                                    return;
                                }
                                jvm_binary.enabled = *val;
                                root.set_launch_defaults(
                                    backend.memory.clone(),
                                    backend.jvm_flags.clone(),
                                    Some(jvm_binary),
                                    cx,
                                );
                            })).into_any_element()
                        })),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::java::defaults::jvm_binary,
                        description: t::settings::java::defaults::jvm_binary_desc,
                        widget: create_jvm_binary_widget(),
                        ..Default::default()
                    },
                ].into(),
                searched_items: None,
            },
        ].into(),
        searched_groups: None,
        custom_render: None,
    }
}

fn create_global_memory_slider_widget() -> SettingItemWidget {
    SettingItemWidget::Backend(Rc::new(|backend, window, cx| {
        let memory = backend.memory.clone().unwrap_or(schema::instance::InstanceMemoryConfiguration {
            enabled: true,
            min: 1024,
            max: 4096,
        });
        let enabled = memory.enabled;
        let total_ram_mb = get_system_ram_mb().max(2048);
        let current_max = if memory.max >= 1024 { memory.max.min(total_ram_mb) } else { 4096.min(total_ram_mb) };

        let mut created = false;
        let slider_state = window.use_keyed_state("default-memory-slider", cx, |_, _| {
            created = true;
            SliderState::new()
                .max(total_ram_mb as f32)
                .min(1024.0)
                .step(256.0)
                .default_value(current_max as f32)
        });

        if created {
            cx.subscribe(&slider_state, |root, _state, event: &SliderEvent, cx| {
                let SliderEvent::Change(change) = event else { return; };
                let val = (change.start() / 256.0).round() * 256.0;
                let new_max = (val as u32).clamp(1024, 131072);

                let Some(backend) = root.backend_config().cloned() else { return; };
                let mut mem = backend.memory.unwrap_or(schema::instance::InstanceMemoryConfiguration {
                    enabled: true,
                    min: 1024,
                    max: 4096,
                });
                if mem.max == new_max {
                    return;
                }
                mem.max = new_max;
                mem.enabled = true;
                root.set_launch_defaults(
                    Some(mem),
                    backend.jvm_flags.clone(),
                    backend.jvm_binary.clone(),
                    cx,
                );
            }).detach();
        } else {
            let state_val = slider_state.read(cx).value().start();
            if (state_val - current_max as f32).abs() > 10.0 {
                slider_state.update(cx, |slider, cx| {
                    slider.set_value(current_max as f32, window, cx);
                });
            }
        }

        v_flex()
            .w(px(320.0))
            .gap_2()
            .px(px(8.0))
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(if enabled { cx.theme().primary } else { cx.theme().muted_foreground })
                            .child(format!("{:.1} GB ({} MB)", current_max as f32 / 1024.0, current_max))
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("System: {:.0} GB", total_ram_mb as f32 / 1024.0))
                    )
            )
            .child(
                Slider::new(&slider_state).disabled(!enabled)
            )
            .child(
                h_flex()
                    .justify_between()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("1.0 GB")
                    .child(format!("{:.0} GB", total_ram_mb as f32 / 1024.0))
            )
            .child(
                h_flex()
                    .gap_1p5()
                    .pt_1()
                    .children(
                        [2048, 4096, 6144, 8192, 12288, 16384]
                            .into_iter()
                            .filter(|&mb| mb <= total_ram_mb)
                            .map(|mb| {
                                let is_active = (current_max as i32 - mb as i32).abs() < 128;
                                let slider_state = slider_state.clone();
                                div()
                                    .id(("global-ram-preset", mb))
                                    .px_2()
                                    .py_0p5()
                                    .text_xs()
                                    .font_medium()
                                    .rounded_none()
                                    .cursor_pointer()
                                    .when_else(is_active, |this| {
                                        this.bg(cx.theme().primary.opacity(0.2))
                                            .text_color(cx.theme().primary)
                                            .border_1()
                                            .border_color(cx.theme().primary)
                                    }, |this| {
                                        this.bg(cx.theme().secondary)
                                            .text_color(cx.theme().foreground)
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .hover(|h| h.bg(cx.theme().secondary_hover))
                                    })
                                    .child(format!("{} GB", mb / 1024))
                                    .on_click(cx.listener(move |root, _, window, cx| {
                                        slider_state.update(cx, |slider, cx| {
                                            slider.set_value(mb as f32, window, cx);
                                        });
                                        let Some(backend) = root.backend_config().cloned() else { return; };
                                        let mut mem = backend.memory.unwrap_or(schema::instance::InstanceMemoryConfiguration {
                                            enabled: true,
                                            min: 1024,
                                            max: 4096,
                                        });
                                        mem.max = mb;
                                        mem.enabled = true;
                                        root.set_launch_defaults(
                                            Some(mem),
                                            backend.jvm_flags.clone(),
                                            backend.jvm_binary.clone(),
                                            cx,
                                        );
                                    }))
                            })
                    )
            )
            .into_any_element()
    }))
}

fn create_jvm_flags_widget() -> SettingItemWidget {
    SettingItemWidget::Backend(Rc::new(|backend, window, cx| {
        let jvm_flags = backend.jvm_flags.clone().unwrap_or_default();
        let mut created = false;
        let state = window.use_keyed_state("default-jvm-flags", cx, |window, cx| {
            created = true;
            let mut state = InputState::new(window, cx);
            state.set_value(jvm_flags.flags.to_string(), window, cx);
            state
        });
        if created {
            cx.subscribe(&state, |root, state, event: &InputEvent, cx| {
                if !matches!(event, InputEvent::Change) {
                    return;
                }
                let Some(backend) = root.backend_config().cloned() else {
                    return;
                };
                let value = state.read(cx).value();
                let mut jvm_flags = backend.jvm_flags.unwrap_or_default();
                if &*jvm_flags.flags == &*value {
                    return;
                }
                jvm_flags.flags = value.into();
                root.set_launch_defaults(
                    backend.memory.clone(),
                    Some(jvm_flags),
                    backend.jvm_binary.clone(),
                    cx,
                );
            }).detach();
        } else {
            let state_read = state.read(cx);
            if &*state_read.value() != &*jvm_flags.flags
                && !state_read.focus_handle(cx).is_focused(window)
            {
                state.update(cx, |state, cx| {
                    state.set_value(jvm_flags.flags.to_string(), window, cx);
                });
            }
        }
        Input::new(&state).w(px(280.0)).disabled(!jvm_flags.enabled).into_any_element()
    }))
}

fn create_jvm_binary_widget() -> SettingItemWidget {
    SettingItemWidget::Backend(Rc::new(|backend, _, cx| {
        let jvm_binary = backend.jvm_binary.clone().unwrap_or_default();
        let path_label = jvm_binary.path.as_ref().map(|path| PathLabel::new(path.clone(), false));
        PathLabel::button_opt(&path_label, "select-default-jvm-binary")
            .disabled(!jvm_binary.enabled)
            .w(px(280.0))
            .min_w_0()
            .flex_shrink(1.0)
            .on_click(cx.listener(|root, _, window, cx| {
                root.select_file(t::settings::java::defaults::select_jvm_binary(), |root, path, cx| {
                    let Some(backend) = root.backend_config().cloned() else {
                        return;
                    };
                    let jvm_binary = InstanceJvmBinaryConfiguration {
                        enabled: backend.jvm_binary.as_ref().is_some_and(|binary| binary.enabled),
                        path,
                    };
                    root.set_launch_defaults(
                        backend.memory.clone(),
                        backend.jvm_flags.clone(),
                        Some(jvm_binary),
                        cx,
                    );
                }, window, cx);
            }))
            .into_any_element()
    }))
}
