use std::{path::Path, sync::{Arc, atomic::AtomicBool}};

use parking_lot::Mutex;

use bridge::{
    handle::BackendHandle,
    install::ContentInstall,
    instance::{InstanceID, InstanceContentID},
    message::{MessageToBackend, QuickPlayLaunch},
    modal_action::ModalAction,
};
use gpui::{prelude::*, *};
use gpui_component::{Root, Theme, WindowExt, scroll::ScrollableElement, v_flex};
use rustc_hash::FxHashSet;

use crate::{Backwards, CloseWindow, Forwards, OpenSettings, entity::DataEntities, game_output::{GameOutput, GameOutputRoot}, interface_config::{InterfaceConfig, LiveGameOutputDisplay}, modals, pages::instance::instance_page::InstanceSubpageType, ui::{LauncherUI, PageType}};

pub struct LauncherRootGlobal {
    pub root: Entity<LauncherRoot>,
}

impl Global for LauncherRootGlobal {}

pub struct LauncherRoot {
    pub ui: Entity<LauncherUI>,
    data: DataEntities,
    focus_handle: FocusHandle,
}

impl LauncherRoot {
    pub fn new(
        data: &DataEntities,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let launcher_ui = cx.new(|cx| LauncherUI::new(data, window, cx));

        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);

        Self {
            ui: launcher_ui,
            data: data.clone(),
            focus_handle,
        }
    }
}

static RENDER_CUSTOM_TITLEBAR: AtomicBool = AtomicBool::new(true);

pub fn should_render_custom_titlebar() -> bool {
    RENDER_CUSTOM_TITLEBAR.load(std::sync::atomic::Ordering::Relaxed)
}

pub(crate)fn set_should_render_custom_titlebar(value: bool) {
    RENDER_CUSTOM_TITLEBAR.store(value, std::sync::atomic::Ordering::Relaxed);
}

impl Render for LauncherRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(message) = &*self.data.panic_messages.deadlock_message.read() {
            let purple = Hsla {
                h: 0.8333333333,
                s: 1.,
                l: 0.25,
                a: 1.,
            };
            return v_flex().size_full().text_color(gpui::white()).bg(purple).child(message.clone()).overflow_y_scrollbar().into_any_element();
        }
        if let Some(message) = &*self.data.panic_messages.panic_message.read() {
            return v_flex().size_full().text_color(gpui::white()).bg(gpui::blue()).child(message.clone()).overflow_y_scrollbar().into_any_element();
        }
        if self.data.backend_handle.is_closed() {
            return v_flex().size_full().text_color(gpui::white()).bg(gpui::red()).child(t::system::backend_shutdown()).into_any_element();
        }

        Theme::global_mut(cx).sheet.margin_top = Pixels::ZERO;

        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        let has_modal = dialog_layer.is_some() || sheet_layer.is_some();
        let ui_content = div()
            .size_full()
            .when(has_modal, |this| this.opacity(0.30))
            .child(self.ui.clone());

        v_flex()
            .size_full()
            .child(ui_content)
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
            .track_focus(&self.focus_handle)
            .on_action(|_: &CloseWindow, window, _| {
                window.remove_window();
            })
            .on_action({
                let data = self.data.clone();
                move |_: &OpenSettings, window, cx| {
                    open_settings_modal(&data, window, cx);
                }
            })
            .on_action({
                let ui = self.ui.clone();
                move |_: &Backwards, window, cx| {
                    ui.update(cx, |ui, cx| {
                        ui.nav_backwards(window, cx);
                    });
                }
            })
            .on_action({
                let ui = self.ui.clone();
                move |_: &Forwards, window, cx| {
                    ui.update(cx, |ui, cx| {
                        ui.nav_forwards(window, cx);
                    });
                }
            })
            .on_mouse_down(MouseButton::Navigate(NavigationDirection::Back), {
                let ui = self.ui.clone();
                move |_, window, cx| {
                    ui.update(cx, |ui, cx| {
                        ui.nav_backwards(window, cx);
                    });
                }
            })
            .on_mouse_down(MouseButton::Navigate(NavigationDirection::Forward), {
                let ui = self.ui.clone();
                move |_, window, cx| {
                    ui.update(cx, |ui, cx| {
                        ui.nav_forwards(window, cx);
                    });
                }
            })
            .into_any_element()
    }
}

pub fn start_new_account_login(
    backend_handle: &BackendHandle,
    window: &mut Window,
    cx: &mut App,
) {
    let modal_action = ModalAction::default();

    backend_handle.send(MessageToBackend::AddNewAccount {
        modal_action: modal_action.clone(),
    });

    let title = t::account::add::title();
    modals::generic::show_modal(window, cx, title.into(), t::account::add::error().into(), modal_action);
}

pub fn start_instance(
    id: InstanceID,
    _name: SharedString,
    quick_play: Option<QuickPlayLaunch>,
    data: &DataEntities,
    window: &mut Window,
    cx: &mut App,
) {
    let modal_action = ModalAction::default();

    // Remove any stale live game outputs
    if let Some(instance_entry) = data.instances.read(cx).entries.get(&id).cloned() {
        instance_entry.update(cx, |entry, _| {
            entry.live_game_output = None;
        });
    };

    let (sender, receiver) = tokio::sync::oneshot::channel();

    let live_game_output_display = InterfaceConfig::get(cx).live_game_output_display;
    let live_game_output = if live_game_output_display == LiveGameOutputDisplay::Hidden {
        None
    } else {
        Some(sender)
    };

    data.backend_handle.send(MessageToBackend::StartInstance {
        id,
        quick_play,
        live_game_output,
        modal_action: modal_action.clone(),
    });

    modals::generic::watch_action_errors(window, cx, t::instance::start::error().into(), modal_action);

    let window_handle = window.window_handle();
    let data = data.clone();
    cx.spawn(async move |cx| {
        let Ok(receiver) = receiver.await else {
            return;
        };

        match live_game_output_display {
            LiveGameOutputDisplay::Hidden => {},
            LiveGameOutputDisplay::SeparateWindow => {
                let options = WindowOptions {
                    app_id: Some("CubeLauncher".into()),
                    window_min_size: Some(size(px(360.0), px(240.0))),
                    titlebar: Some(TitlebarOptions {
                        title: Some(t::system::game_output().into()),
                        ..Default::default()
                    }),
                    window_decorations: Some(WindowDecorations::Server),
                    ..Default::default()
                };
                _ = cx.open_window(options, |window, cx| {
                    let game_output = cx.new(|cx| GameOutput::new(receiver, cx));
                    let game_output_root = cx.new(|cx| GameOutputRoot::new(game_output.clone(), window, cx));
                    window.activate_window();
                    cx.new(|cx| Root::new(game_output_root, window, cx))
                });
            },
            LiveGameOutputDisplay::TabOnInstancePage => {
                _ = cx.update_window(window_handle, |_, window, cx| {
                    let game_output = cx.new(|cx| GameOutput::new(receiver, cx));
                    let game_output_root = cx.new(|cx| GameOutputRoot::new(game_output.clone(), window, cx));

                    let Some(instance_entry) = data.instances.read(cx).entries.get(&id).cloned() else {
                        return;
                    };

                    instance_entry.update(cx, |entry, _| {
                        entry.live_game_output = Some(game_output_root);
                    });

                    let config = InterfaceConfig::get(cx);
                    if matches!(config.main_page, PageType::InstancePage { .. }) && config.instance_subpage != InstanceSubpageType::LiveGameOutput {
                        InterfaceConfig::get_mut(cx).instance_subpage = InstanceSubpageType::LiveGameOutput;
                    }
                });
            },
        }
    }).detach();
}


pub fn start_install(
    content_install: ContentInstall,
    name: SharedString,
    icon: Option<schema::unique_bytes::UniqueBytes>,
    is_modpack: bool,
    data: &DataEntities,
    _window: &mut Window,
    cx: &mut App,
) {
    let modal_action = ModalAction::default();

    let instance_id = match &content_install.target {
        bridge::install::InstallTarget::Instance(id) => Some(*id),
        bridge::install::InstallTarget::NewInstance { name: target_name } => {
            let reg_name = target_name.as_ref().map(|s| s.to_string()).unwrap_or_else(|| name.to_string());
            data.tasks.update(cx, |t, cx| {
                t.register_pending_creation(reg_name, icon.clone(), modal_action.clone(), is_modpack, cx);
            });
            None
        },
        bridge::install::InstallTarget::Library => None,
    };

    if !matches!(&content_install.target, bridge::install::InstallTarget::NewInstance { .. }) {
        data.tasks.update(cx, |tasks, cx| {
            tasks.add_task(instance_id, name.clone(), icon.clone(), modal_action.clone(), is_modpack, cx);
        });
    }

    data.backend_handle.send(MessageToBackend::InstallContent {
        content: content_install.clone(),
        modal_action: modal_action.clone(),
    });
}

pub fn change_mod_version(
    content_install: ContentInstall,
    hash: u64,
    updating: &Arc<Mutex<FxHashSet<u64>>>,
    name: SharedString,
    icon: Option<schema::unique_bytes::UniqueBytes>,
    data: &DataEntities,
    _window: &mut Window,
    cx: &mut App,
) {
    let modal_action = ModalAction::default();

    updating.lock().insert(hash);
    modal_action.add_finish_effect({
        let updating = updating.clone();
        move || {
            updating.lock().remove(&hash);
        }
    });

    let instance_id = match &content_install.target {
        bridge::install::InstallTarget::Instance(id) => Some(*id),
        _ => None,
    };

    data.tasks.update(cx, |tasks, cx| {
        tasks.add_task(instance_id, name.clone(), icon.clone(), modal_action.clone(), false, cx);
    });

    data.backend_handle.send(MessageToBackend::InstallContent {
        content: content_install.clone(),
        modal_action: modal_action.clone(),
    });
}

pub fn start_update_check(
    instance: InstanceID,
    backend_handle: &BackendHandle,
    window: &mut Window,
    cx: &mut App,
) {
    let modal_action = ModalAction::default();

    backend_handle.send(MessageToBackend::UpdateCheck {
        instance,
        modal_action: modal_action.clone(),
    });

    let title: SharedString = t::instance::content::update::check::error().into();
    modals::generic::watch_action_errors(window, cx, title, modal_action);
}

pub fn update_single_mod(
    instance: InstanceID,
    mod_id: InstanceContentID,
    hash: u64,
    updating: &Arc<Mutex<FxHashSet<u64>>>,
    name: SharedString,
    data: &DataEntities,
    _window: &mut Window,
    cx: &mut App,
) {
    let modal_action = ModalAction::default();

    updating.lock().insert(hash);
    modal_action.add_finish_effect({
        let updating = updating.clone();
        move || {
            updating.lock().remove(&hash);
        }
    });

    let (inst_name, inst_icon) = data.instances.read(cx).entries.get(&instance).map(|e| {
        let entry = e.read(cx);
        (entry.name.clone(), entry.icon.clone())
    }).unwrap_or_else(|| (name.clone(), None));

    data.tasks.update(cx, |tasks, cx| {
        tasks.add_task(Some(instance), inst_name, inst_icon, modal_action.clone(), false, cx);
    });

    data.backend_handle.send(MessageToBackend::UpdateContent {
        instance,
        content_id: mod_id,
        modal_action: modal_action.clone(),
    });
}

pub fn upload_log_file(
    path: Arc<Path>,
    backend_handle: &BackendHandle,
    window: &mut Window,
    cx: &mut App,
) {
    let modal_action = ModalAction::default();

    backend_handle.send(MessageToBackend::UploadLogFile {
        path,
        modal_action: modal_action.clone(),
    });

    let title: SharedString = t::instance::logs::upload::title().into();
    modals::generic::show_modal(window, cx, title, t::instance::logs::upload::error().into(), modal_action);
}

pub fn switch_page(
    page: PageType,
    breadcrumbs: &[PageType],
    window: &mut Window,
    cx: &mut App,
) {
    cx.update_global::<LauncherRootGlobal, ()>(|global, cx| {
        global.root.update(cx, |launcher_root, cx| {
            launcher_root.ui.update(cx, |ui, cx| {
                ui.switch_page(page, breadcrumbs, window, cx);
            });
        });
    });
}

pub fn open_settings_modal(data: &DataEntities, window: &mut Window, cx: &mut App) {
    capture_backdrop_blur(window, cx);
    let settings_page = crate::settings::create_settings_page(data, window, cx);
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .w(px(940.0))
            .h(px(580.0))
            .margin_top(px(32.0))
            .title(t::settings::title())
            .close_button(true)
            .overlay_closable(true)
            .child(settings_page.clone())
    });
}

pub fn close_settings_modal(cx: &mut App) {
    if let Some(active) = cx.active_window() {
        cx.update_window(active, |_, window, cx| {
            window.close_dialog(cx);
        }).ok();
    }
}

pub fn capture_backdrop_blur(_window: &mut Window, _cx: &mut App) {
    // Backdrop blur capture hook for modals
}
