use std::sync::{Arc, atomic::AtomicBool};

use bridge::{instance::InstanceStatus, message::{BridgeNotificationType, MessageToFrontend}, quit::QuitCoordinator};
use gpui::{AnyWindowHandle, App, SharedString, Window};
use gpui_component::{notification::{Notification, NotificationType}, Root, WindowExt};

use crate::{entity::{DataEntities, account::AccountEntries, instance::{ContentStates, InstanceEntries}, metadata::FrontendMetadata}, interface_config::InterfaceConfig, root::LauncherRoot};

pub struct Processor {
    data: DataEntities,
    main_window_handle: Option<AnyWindowHandle>,
    main_window_hidden: Arc<AtomicBool>,
    waiting_for_window: Vec<MessageToFrontend>,
    quit_coordinator: QuitCoordinator,
    is_first_launch: bool,
    pub self_entity: Option<gpui::Entity<Processor>>,
}

impl Processor {
    pub fn new(data: DataEntities, main_window_hidden: Arc<AtomicBool>, quit_coordinator: QuitCoordinator) -> Self {
        Self {
            data,
            main_window_handle: None,
            main_window_hidden,
            waiting_for_window: Vec::new(),
            quit_coordinator,
            is_first_launch: true,
            self_entity: None,
        }
    }

    pub fn set_main_window_handle(&mut self, window: AnyWindowHandle, cx: &mut App) {
        self.main_window_handle = Some(window);
        self.process_messages_waiting_for_window(cx);
    }

    pub fn process_messages_waiting_for_window(&mut self, cx: &mut App) {
        for message in std::mem::take(&mut self.waiting_for_window) {
            self.process(message, cx);
        }
    }

    #[inline(always)]
    pub fn with_main_window(&mut self, message: MessageToFrontend, cx: &mut App, func: impl FnOnce(&mut Processor, MessageToFrontend, &mut Window, &mut App)) {
        let Some(handle) = self.main_window_handle else {
            self.waiting_for_window.push(message);
            return;
        };

        _ = handle.update(cx, |_, window, cx| {
            (func)(self, message, window, cx);
        });
    }

    pub fn process(&mut self, message: MessageToFrontend, cx: &mut App) {
        match message {
            MessageToFrontend::AccountsUpdated {
                accounts,
                selected_account,
            } => {
                AccountEntries::set(&self.data.accounts, accounts, selected_account, cx);
            },
            MessageToFrontend::InstanceAdded {
                id,
                name,
                icon,
                root_path,
                dot_minecraft_folder,
                configuration,
                playtime,
                worlds_state,
                servers_state,
                content_states
            } => {
                InstanceEntries::add(
                    &self.data.instances,
                    id,
                    name.as_str().into(),
                    icon.clone(),
                    root_path,
                    dot_minecraft_folder,
                    configuration,
                    playtime,
                    worlds_state,
                    servers_state,
                    ContentStates::new(id, content_states, self.data.backend_handle.clone()),
                    cx,
                );

                if let Some(modal_action) = self.data.tasks.update(cx, |tasks, cx| {
                    tasks.take_pending_creation(&name, id, icon.clone(), cx)
                }) {
                    self.data.backend_handle.send(bridge::message::MessageToBackend::PrepareInstanceFiles {
                        id,
                        modal_action,
                    });
                }
            },
            MessageToFrontend::InstanceRemoved { id } => {
                InstanceEntries::remove(&self.data.instances, id, cx);
            },
            MessageToFrontend::InstanceModified {
                id,
                name,
                icon,
                root_path,
                dot_minecraft_folder,
                configuration,
                playtime,
                status,
            } => {
                if status == InstanceStatus::Running {
                    let behavior = {
                        let cfg = InterfaceConfig::get(cx);
                        if cfg.hide_main_window_on_launch && cfg.game_launch_behavior == crate::interface_config::GameLaunchBehavior::KeepOpen {
                            crate::interface_config::GameLaunchBehavior::HideToTray
                        } else {
                            cfg.game_launch_behavior
                        }
                    };

                    match behavior {
                        crate::interface_config::GameLaunchBehavior::CloseLauncher => {
                            cx.quit();
                        },
                        crate::interface_config::GameLaunchBehavior::HideToTray => {
                            if let Some(handle) = self.main_window_handle.take() {
                                self.main_window_hidden.store(true, std::sync::atomic::Ordering::SeqCst);
                                _ = handle.update(cx, |_, window, _| {
                                    window.remove_window();
                                });
                            }
                        },
                        crate::interface_config::GameLaunchBehavior::KeepOpen => {},
                    }
                } else if status == InstanceStatus::NotRunning {
                    if self.main_window_handle.is_none() && self.main_window_hidden.load(std::sync::atomic::Ordering::SeqCst) {
                        self.quit_coordinator.set_can_quit(false);
                        self.main_window_handle = Some(crate::open_main_window(&self.data, cx));
                        self.main_window_hidden.store(false, std::sync::atomic::Ordering::SeqCst);
                        self.process_messages_waiting_for_window(cx);
                    }
                }

                InstanceEntries::modify(
                    &self.data.instances,
                    id,
                    name.as_str().into(),
                    icon.clone(),
                    root_path,
                    dot_minecraft_folder,
                    configuration,
                    playtime,
                    status,
                    cx,
                );
            },
            MessageToFrontend::InstancePlaytimeUpdated { id, playtime } => {
                InstanceEntries::set_playtime(&self.data.instances, id, playtime, cx);
            },
            MessageToFrontend::InstanceWorldsUpdated { id, worlds } => {
                InstanceEntries::set_worlds(&self.data.instances, id, worlds, cx);
            },
            MessageToFrontend::InstanceServersUpdated { id, servers } => {
                InstanceEntries::set_servers(&self.data.instances, id, servers, cx);
            },
            MessageToFrontend::InstanceContentUpdated { id, content_folder, content } => {
                InstanceEntries::set_content(&self.data.instances, id, content_folder, content, cx);
            },
            MessageToFrontend::AddNotification { .. } => {
                self.with_main_window(message, cx, |_, message, window, cx| {
                    let MessageToFrontend::AddNotification { notification_type, message } = message else {
                        unreachable!();
                    };

                    let notification_type = match notification_type {
                        BridgeNotificationType::Success => NotificationType::Success,
                        BridgeNotificationType::Info => NotificationType::Info,
                        BridgeNotificationType::Error => NotificationType::Error,
                        BridgeNotificationType::Warning => NotificationType::Warning,
                    };
                    let mut notification: Notification = (notification_type, SharedString::from(message)).into();
                    if let NotificationType::Error = notification_type {
                        notification = notification.autohide(false);
                    }
                    window.push_notification(notification, cx);
                });
            },
            MessageToFrontend::Refresh => {
                let Some(handle) = self.main_window_handle else {
                    return;
                };
                _ = handle.update(cx, |_, window, _| {
                    window.refresh();
                });
            },
            MessageToFrontend::Quit => {
                cx.quit();
            },
            MessageToFrontend::CloseModal => {
                let Some(handle) = self.main_window_handle else {
                    return;
                };
                _ = handle.update(cx, |_, window, cx| {
                    window.close_all_dialogs(cx);
                });
            },
            MessageToFrontend::MoveInstanceToTop { id } => {
                InstanceEntries::move_to_top(&self.data.instances, id, cx);
            },
            MessageToFrontend::MetadataResult { request, result, keep_alive_handle } => {
                FrontendMetadata::set(&self.data.metadata, request, result, keep_alive_handle, cx);
            },
            MessageToFrontend::SkinLibraryUpdated { skin_library } => {
                self.data.set_skin_library(skin_library, cx);
            },
            MessageToFrontend::UpdateAvailable { .. } => {
                self.with_main_window(message, cx, |_, message, window, cx| {
                    let MessageToFrontend::UpdateAvailable { update } = message else {
                        unreachable!();
                    };

                    if let Some(root) = window.root::<Root>().flatten() {
                        if let Ok(launcher_root) = root.read(cx).view().clone().downcast::<LauncherRoot>() {
                            launcher_root.update(cx, |launcher_root, cx| {
                                launcher_root.ui.update(cx, |ui, cx| {
                                    ui.update = Some(update);
                                    cx.notify();
                                });
                            });
                        }
                    }
                });
            },
            MessageToFrontend::ManualCurseforgeDownloadsRequired { request: _ } => {
                self.with_main_window(message, cx, |_processor, message, window, cx| {
                    let MessageToFrontend::ManualCurseforgeDownloadsRequired { request } = message else { unreachable!() };
                    crate::modals::manual_curseforge_downloads::open(request, window, cx);
                });
            },
            MessageToFrontend::OpenOrFocusMainWindow => {
                self.quit_coordinator.set_can_quit(false);

                if let Some(handle) = self.main_window_handle {
                    let res = handle.update(cx, |_, window, _| {
                        window.activate_window();
                    });
                    if res.is_ok() {
                        return;
                    }
                }

                if self.is_first_launch {
                    self.is_first_launch = false;
                    let (splash_handle, splash_is_alive) = crate::splash::open_splash_window(cx);
                    let data = self.data.clone();
                    let self_entity = self.self_entity.clone();

                    cx.spawn({
                        let splash_is_alive = splash_is_alive.clone();
                        async move |cx| {
                            cx.background_executor().timer(std::time::Duration::from_millis(1300)).await;

                            _ = cx.update(|cx| {
                                let main_handle = crate::open_main_window(&data, cx);
                                if let Some(self_entity) = self_entity {
                                    self_entity.update(cx, |p, cx| {
                                        p.main_window_handle = Some(main_handle);
                                        p.main_window_hidden.store(false, std::sync::atomic::Ordering::SeqCst);
                                        p.process_messages_waiting_for_window(cx);
                                    });
                                }

                                splash_is_alive.store(false, std::sync::atomic::Ordering::SeqCst);
                                _ = splash_handle.update(cx, |_, window, _| {
                                    window.remove_window();
                                });
                            });
                        }
                    }).detach();
                    return;
                }

                self.main_window_handle = Some(crate::open_main_window(&self.data, cx));
                self.main_window_hidden.store(false, std::sync::atomic::Ordering::SeqCst);
                self.process_messages_waiting_for_window(cx);
            }
        }
    }
}
