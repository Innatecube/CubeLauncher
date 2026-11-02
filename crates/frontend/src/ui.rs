use std::{collections::VecDeque, sync::Arc};

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme as _, InteractiveElementExt, StyledExt, WindowExt, button::{Button, ButtonVariants}, h_flex, notification::{Notification, NotificationType}, scroll::ScrollableElement, tooltip::Tooltip, v_flex
};
use bridge::message::MessageToBackend;
use rustc_hash::FxHashMap;
use schema::pandora_update::UpdatePrompt;
use serde::{Deserialize, Serialize};

use crate::{
    component::{
        generic_title_bar::TitleBarState,
        main_title_bar::MainTitleBar,
        page_path::PagePath,
        reorderable::{Reorderable, ReorderableDragInfo, ReorderableState},
    },
    entity::{
        DataEntities,
        account::AccountExt,
        instance::{
            InstanceAddedEvent, InstanceEntries, InstanceEntry, InstanceModifiedEvent,
            InstanceMovedToTopEvent, InstanceRemovedEvent,
        },
    },
    icon::PandoraIcon,
    interface_config::InterfaceConfig,
    pages::{
        curseforge_page::CurseforgeSearchPage, import::ImportPage,
        instance::instance_page::InstancePage, instances_page::InstancesPage,
        modrinth_page::ModrinthSearchPage, modrinth_project_page::ModrinthProjectPage,
        page::Page, skins_page::SkinsPage, syncing_page::SyncingPage,
    },
    png_render_cache,
};

pub struct LauncherUI {
    pub data: DataEntities,
    pub page: LauncherPage,
    pub update: Option<UpdatePrompt>,
    pub is_account_flyout_open: bool,
    pub account_flyout_closed_at: Option<std::time::Instant>,
    sidebar_reorderable_state: Entity<ReorderableState>,
    page_history_backwards: VecDeque<(PageType, Arc<[PageType]>)>,
    page_history_forwards: Vec<(PageType, Arc<[PageType]>)>,
    previous_pages: FxHashMap<PageType, LauncherPage>,
    pending_page: Option<(PageType, Arc<[PageType]>)>,
    _instance_added_subscription: Subscription,
    _instance_modified_subscription: Subscription,
    _instance_removed_subscription: Subscription,
    _instance_moved_to_top_subscription: Subscription,
    _tasks_subscription: Subscription,
}

pub const ICONIZED_SIDEBAR_WIDTH: f32 = 64.0;
pub const MIN_SIDEBAR_WIDTH: f32 = 180.0;
pub const MAX_SIDEBAR_WIDTH: f32 = 300.0;
pub const DEFAULT_SIDEBAR_WIDTH: f32 = 220.0;

#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PageType {
    #[default]
    Instances,
    Skins,
    Modrinth {
        installing_for: Option<SharedString>,
    },
    Curseforge {
        installing_for: Option<SharedString>,
    },
    Import,
    Syncing,
    ModrinthProject {
        project_id: SharedString,
        project_title: SharedString,
        install_for: Option<SharedString>,
    },
    InstancePage {
        name: SharedString,
    },
    Settings,
}

impl PageType {
    pub fn title(&self, data: &DataEntities, cx: &App) -> SharedString {
        match self {
            PageType::Instances => t::instance::title().into(),
            PageType::Skins => "Wardrobe".into(),
            PageType::Modrinth { installing_for } => {
                if installing_for.is_some() {
                    "install content".into()
                } else {
                    t::modrinth::name().into()
                }
            },
            PageType::Curseforge { installing_for } => {
                if installing_for.is_some() {
                    "install content".into()
                } else {
                    t::curseforge::name().into()
                }
            },
            PageType::Import => t::import::label().into(),
            PageType::Syncing => t::instance::sync::label().into(),
            PageType::ModrinthProject { project_title, .. } => project_title.clone(),
            PageType::InstancePage { name } => {
                InstanceEntries::find_title_by_name(&data.instances, name, cx)
                    .unwrap_or_else(|| name.clone())
            },
            PageType::Settings => t::settings::title().into(),
        }
    }
}

#[derive(Clone)]
pub enum LauncherPage {
    Instances(Entity<InstancesPage>),
    Skins(Entity<SkinsPage>),
    Modrinth(Entity<ModrinthSearchPage>),
    Curseforge(Entity<CurseforgeSearchPage>),
    Import(Entity<ImportPage>),
    Syncing(Entity<SyncingPage>),
    ModrinthProject(Entity<ModrinthProjectPage>),
    InstancePage(Entity<InstancePage>),
    Settings(Entity<crate::settings::SettingsRoot>),
}

impl LauncherPage {
    fn render(self, ui: &LauncherUI, window: &mut Window, cx: &mut App) -> impl IntoElement {
        fn process(entity: Entity<impl Page>, window: &mut Window, cx: &mut App) -> (bool, AnyElement, AnyElement) {
            entity.update(cx, |page, cx| {
                (page.scrollable(cx), page.controls(window, cx).into_any_element(), page.render(window, cx).into_any_element())
            })
        }

        let (scrollable, controls, page) = match self {
            LauncherPage::Instances(entity) => process(entity, window, cx),
            LauncherPage::Skins(entity) => process(entity, window, cx),
            LauncherPage::Modrinth(entity) => process(entity, window, cx),
            LauncherPage::Curseforge(entity) => process(entity, window, cx),
            LauncherPage::Import(entity) => process(entity, window, cx),
            LauncherPage::Syncing(entity) => process(entity, window, cx),
            LauncherPage::ModrinthProject(entity) => process(entity, window, cx),
            LauncherPage::InstancePage(entity) => process(entity, window, cx),
            LauncherPage::Settings(entity) => process(entity, window, cx),
        };

        let config = InterfaceConfig::get(cx);
        let page_path = PagePath::new(ui.data.clone(), config.main_page.clone(), config.page_path.clone());
        let title_bar = MainTitleBar {
            page_path,
            controls,
            update: ui.update.clone(),
            send: ui.data.backend_handle.clone(),
            tasks: ui.data.tasks.clone(),
        };

        let is_flyout_open = ui.data.tasks.read(cx).is_flyout_open;
        let flyout_element = if is_flyout_open {
            Some(ui.render_download_flyout(cx))
        } else {
            None
        };

        let content = if scrollable {
            v_flex()
                .size_full()
                .child(title_bar)
                .child(div().w_full().flex_1().min_h_0().overflow_hidden().child(
                    v_flex().size_full().overflow_y_scrollbar().child(page),
                ))
        } else {
            v_flex()
                .size_full()
                .child(title_bar)
                .child(div().w_full().flex_1().min_h_0().overflow_hidden().child(page))
        };

        v_flex()
            .size_full()
            .relative()
            .child(content)
            .children(flyout_element)
    }
}

impl LauncherUI {
    pub fn new(data: &DataEntities, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _instance_added_subscription =
            cx.subscribe::<_, InstanceAddedEvent>(&data.instances, |_, _, _, cx| {
                cx.notify();
            });
        let _instance_modified_subscription =
            cx.subscribe_in::<_, InstanceModifiedEvent>(&data.instances, window, |this, _, event, window, cx| {
                if let LauncherPage::InstancePage(page) = &this.page
                    && page.read(cx).instance.read(cx).id == event.instance.id
                {
                    let page_path = InterfaceConfig::get_mut(cx).page_path.clone();
                    this.switch_page(PageType::InstancePage { name: event.instance.name.clone() }, &*page_path, window, cx);
                }
                cx.notify();
            });
        let _instance_removed_subscription =
            cx.subscribe_in::<_, InstanceRemovedEvent>(&data.instances, window, |this, _, event, window, cx| {
                if let LauncherPage::InstancePage(page) = &this.page
                    && page.read(cx).instance.read(cx).id == event.id
                {
                    this.switch_page(PageType::Instances, &[], window, cx);
                }
                cx.notify();
            });
        let _instance_moved_to_top_subscription =
            cx.subscribe::<_, InstanceMovedToTopEvent>(&data.instances, |_, _, _, cx| {
                cx.notify();
            });
        let _tasks_subscription = cx.observe(&data.tasks, |_, _, cx| {
            cx.notify();
        });

        let sidebar_reorderable_state = ReorderableState::new(window, cx);

        let config = InterfaceConfig::get(cx);
        let main_page = config.main_page.clone();
        let original_page_path = config.page_path.clone();

        // If main_page failed to deserialize, also reset the path
        if main_page == PageType::Instances {
            let config = InterfaceConfig::get_mut(cx);
            config.page_path = [].into();
        }

        let mut pending_page = None;

        let page = match Self::create_page(&data, main_page.clone(), window, cx) {
            Ok(page) => page,
            Err(page_type) => {
                pending_page = Some((main_page, original_page_path));

                let config = InterfaceConfig::get_mut(cx);
                config.main_page = page_type.clone();
                config.page_path = [].into();
                Self::create_page(&data, page_type, window, cx).unwrap()
            },
        };

        Self {
            data: data.clone(),
            page,
            update: None,
            is_account_flyout_open: false,
            account_flyout_closed_at: None,
            sidebar_reorderable_state,
            page_history_backwards: VecDeque::with_capacity(32),
            page_history_forwards: Vec::new(),
            previous_pages: FxHashMap::default(),
            pending_page,
            _instance_added_subscription,
            _instance_modified_subscription,
            _instance_removed_subscription,
            _instance_moved_to_top_subscription,
            _tasks_subscription,
        }
    }

    fn create_page(data: &DataEntities, page: PageType, window: &mut Window, cx: &mut Context<Self>) -> Result<LauncherPage, PageType> {
        match page {
            PageType::Instances => {
                Ok(LauncherPage::Instances(cx.new(|cx| InstancesPage::new(data, window, cx))))
            },
            PageType::Skins => {
                Ok(LauncherPage::Skins(cx.new(|cx| SkinsPage::new(data, window, cx))))
            },
            PageType::Modrinth { installing_for } => {
                let installing_for = installing_for.as_ref().map(|name| InstanceEntries::find_id_by_name(&data.instances, name, cx));

                if let Some(None) = installing_for {
                    return Err(PageType::Modrinth { installing_for: None })
                }

                let page = cx.new(|cx| {
                    ModrinthSearchPage::new(installing_for.flatten(), data, window, cx)
                });
                Ok(LauncherPage::Modrinth(page))
            },
            PageType::Curseforge { installing_for } => {
                let installing_for = installing_for.as_ref().map(|name| InstanceEntries::find_id_by_name(&data.instances, name, cx));

                if let Some(None) = installing_for {
                    return Err(PageType::Curseforge { installing_for: None })
                }

                let page = cx.new(|cx| {
                    CurseforgeSearchPage::new(installing_for.flatten(), data, window, cx)
                });
                Ok(LauncherPage::Curseforge(page))
            },
            PageType::Import => {
                Ok(LauncherPage::Import(cx.new(|cx| ImportPage::new(data, window, cx))))
            },
            PageType::Syncing => {
                Ok(LauncherPage::Syncing(cx.new(|cx| SyncingPage::new(data, window, cx))))
            },
            PageType::ModrinthProject { project_id, install_for, project_title } => {
                let install_for_id = install_for.as_ref().map(|name| InstanceEntries::find_id_by_name(&data.instances, name, cx));

                if let Some(None) = install_for_id {
                    return Err(PageType::ModrinthProject { project_id, install_for: None, project_title })
                }

                let project_id = project_id.clone();
                let page = cx.new(|cx| {
                    ModrinthProjectPage::new(project_id, install_for_id.flatten(), data, window, cx,)
                });
                Ok(LauncherPage::ModrinthProject(page))
            },
            PageType::InstancePage { ref name } => {
                let Some(id) = InstanceEntries::find_id_by_name(&data.instances, name, cx) else {
                    return Err(PageType::Instances);
                };

                Ok(LauncherPage::InstancePage(cx.new(|cx| {
                    InstancePage::new(id, data, window, cx)
                })))
            },
            PageType::Settings => {
                let settings = crate::settings::create_settings_page(data, window, cx);
                Ok(LauncherPage::Settings(settings))
            },
        }
    }

    pub fn switch_page(&mut self, page: PageType, page_path: &[PageType], window: &mut Window, cx: &mut Context<Self>) {
        let page_path: Arc<[PageType]> = page_path.into();

        let config = InterfaceConfig::get(cx);
        if config.main_page == page {
            return;
        }

        self.page_history_forwards.clear();
        if self.page_history_backwards.len() >= 32 {
            self.page_history_backwards.pop_back();
        }
        self.page_history_backwards.push_front((config.main_page.clone(), config.page_path.clone()));

        self.switch_page_without_history(page, page_path, window, cx);
    }

    fn switch_page_without_history(&mut self, page: PageType, page_path: Arc<[PageType]>, window: &mut Window, cx: &mut Context<Self>) {
        self.pending_page = None;

        let config = InterfaceConfig::get_mut(cx);
        let previous_page_type = std::mem::replace(&mut config.main_page, page.clone());
        config.main_page = page.clone();
        config.page_path = page_path.clone();

        if let Some(previous_page) = self.previous_pages.remove(&page) {
            self.page = previous_page;
            self.previous_pages.retain(|k, _| page_path.contains(k));
            cx.notify();
            return;
        }

        match Self::create_page(&self.data, page, window, cx) {
            Ok(page) => {
                let previous_page = std::mem::replace(&mut self.page, page);
                if page_path.contains(&previous_page_type) {
                    self.previous_pages.insert(previous_page_type, previous_page);
                }
                self.previous_pages.retain(|k, _| page_path.contains(k));
            },
            Err(fallback) => {
                let config = InterfaceConfig::get_mut(cx);
                config.main_page = fallback.clone();
                config.page_path = [].into();
                self.previous_pages.clear();
                self.page = Self::create_page(&self.data, fallback, window, cx).unwrap();
            },
        }

        cx.notify();
    }

    pub fn nav_backwards(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((page, page_path)) = self.page_history_backwards.pop_front() else {
            return;
        };

        let config = InterfaceConfig::get(cx);
        self.page_history_forwards.push((config.main_page.clone(), config.page_path.clone()));

        self.switch_page_without_history(page, page_path, window, cx);
    }

    pub fn nav_forwards(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((page, page_path)) = self.page_history_forwards.pop() else {
            return;
        };

        let config = InterfaceConfig::get(cx);
        self.page_history_backwards.push_front((config.main_page.clone(), config.page_path.clone()));

        self.switch_page_without_history(page, page_path, window, cx);
    }

    pub fn render_download_flyout(&self, cx: &mut App) -> AnyElement {
        let snapshots = self.data.tasks.read(cx).snapshots();
        let active_snapshots: Vec<_> = snapshots.iter().filter(|s| !s.is_finished && s.is_modpack).cloned().collect();
        let tasks_entity = self.data.tasks.clone();
        let right_pos = if cfg!(target_os = "macos") { px(16.0) } else { px(35.0) };

        if active_snapshots.is_empty() {
            let empty_right = if cfg!(target_os = "macos") { px(16.0) } else { px(70.0) };
            return div()
                .id("download-flyout-empty")
                .occlude()
                .absolute()
                .top(px(52.0))
                .right(empty_right)
                .w(px(260.0))
                .bg(rgb(0x0b1e1e))
                .border_1()
                .border_color(rgb(0x163232))
                .rounded_none()
                .shadow_2xl()
                .p_4()
                .on_mouse_down(MouseButton::Left, |_, window, cx| {
                    window.prevent_default();
                    cx.stop_propagation();
                })
                .on_mouse_down_out(move |_, _, cx| {
                    tasks_entity.update(cx, |t, cx| t.close_flyout(cx));
                })
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .child(div().size_6().text_color(rgb(0x00e5ff)).child(PandoraIcon::CircleCheck))
                .child(div().text_xs().font_medium().text_color(rgb(0x94a3b8)).child("No active downloads"))
                .into_any_element();
        }

        let mut task_cards = Vec::new();
        for (i, task) in active_snapshots.iter().take(3).enumerate() {
            let icon_view = if let Some(icon) = &task.icon {
                let resize = png_render_cache::ImageTransformation::Resize { width: 44, height: 44 };
                png_render_cache::render_with_transform(icon.clone(), resize, cx)
                    .size(px(44.0))
                    .rounded_none()
                    .into_any_element()
            } else {
                crate::component::instance_icon::render_instance_tile(
                    None,
                    &task.name,
                    Some("icons/box.svg"),
                    44.0,
                    10.0,
                    cx,
                )
            };

            let raw_pct = if task.total > 0 {
                (task.count as f32 / task.total as f32).clamp(0.0, 1.0)
            } else if task.count > 0 {
                0.15
            } else {
                0.05
            };
            let pct = if task.is_finished {
                1.0
            } else {
                raw_pct.min(0.98)
            };
            let pct_int = (pct * 100.0) as u32;

            let mb_downloaded = task.count as f64 / 1_048_576.0;
            let mb_total = task.total as f64 / 1_048_576.0;
            let mb_text = if task.is_finished {
                "Complete".to_string()
            } else if task.total > 0 {
                if task.speed_mb_s > 0.05 {
                    format!("{:.1} / {:.0} MB • {:.1} MB/s", mb_downloaded, mb_total, task.speed_mb_s)
                } else {
                    format!("{:.1} / {:.0} MB", mb_downloaded, mb_total)
                }
            } else if task.count > 0 {
                format!("{:.1} MB", mb_downloaded)
            } else {
                task.stage.to_string()
            };

            let fill_w = (266.0 * pct).clamp(4.0, 266.0);

            if i > 0 {
                task_cards.push(
                    div()
                        .w_full()
                        .h(px(1.0))
                        .bg(rgb(0x1e2433))
                        .my_1()
                        .into_any_element(),
                );
            }

            task_cards.push(
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_3()
                    // 44x44 icon
                    .child(
                        div()
                            .size(px(44.0))
                            .min_w(px(44.0))
                            .max_w(px(44.0))
                            .rounded_none()
                            .overflow_hidden()
                            .flex_none()
                            .child(icon_view)
                    )
                    // 266px details
                    .child(
                        v_flex()
                            .w(px(266.0))
                            .min_w(px(266.0))
                            .max_w(px(266.0))
                            .gap_1p5()
                            // Name row
                            .child(
                                div()
                                    .w(px(266.0))
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(rgb(0xf8fafc))
                                    .truncate()
                                    .child(task.name.clone())
                            )
                            // Stage subtitle
                            .child(
                                div()
                                    .w(px(266.0))
                                    .text_xs()
                                    .text_color(rgb(0x94a3b8))
                                    .truncate()
                                    .child(task.stage.to_string())
                            )
                            // Progress bar (266px width, 6px height)
                            .child(
                                div()
                                    .w(px(266.0))
                                    .h(px(6.0))
                                    .rounded_none()
                                    .bg(rgb(0x1e2433))
                                    .relative()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .absolute()
                                            .top_0()
                                            .left_0()
                                            .h_full()
                                            .w(px(fill_w))
                                            .rounded_none()
                                            .bg(rgb(0x00e5ff))
                                    )
                            )
                            // MB / Speed + percentage row
                            .child(
                                h_flex()
                                    .w(px(266.0))
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(rgb(0x94a3b8))
                                            .child(mb_text)
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .font_bold()
                                            .text_color(rgb(0x00e5ff))
                                            .child(format!("{}%", pct_int))
                                    )
                            )
                    )
                    .into_any_element(),
            );
        }

        div()
            .id("download-flyout-card")
            .occlude()
            .absolute()
            .top(px(52.0))
            .right(right_pos)
            .w(px(350.0))
            .bg(rgb(0x0b1e1e))
            .border_1()
            .border_color(rgb(0x163232))
            .rounded_none()
            .shadow_2xl()
            .p(px(14.0))
            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_mouse_down_out(move |_, _, cx| {
                tasks_entity.update(cx, |t, cx| t.close_flyout(cx));
            })
            .child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .children(task_cards)
            )
            .into_any_element()
    }

    fn render_account_flyout(&self, _iconized_sidebar: bool, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let accounts_entity = self.data.accounts.read(cx);
        let accounts = accounts_entity.accounts.clone();
        let selected_uuid = accounts_entity.selected_account_uuid;
        let hide_skins = InterfaceConfig::get(cx).hide_skins;
        let hide_usernames = InterfaceConfig::get(cx).hide_usernames;
        let backend_handle = self.data.backend_handle.clone();

        let add_btn = {
            let backend_handle = backend_handle.clone();
            h_flex()
                .id("flyout-add-account")
                .w_full()
                .h(px(32.0))
                .px_3()
                .gap_2()
                .items_center()
                .justify_center()
                .rounded_none()
                .cursor_pointer()
                .bg(rgb(0x04242e))
                .border_1()
                .border_color(rgb(0x00e5ff).opacity(0.4))
                .text_color(rgb(0x00f0ff))
                .font_family(crate::interface_config::DEFAULT_FONT)
                .text_size(px(12.0))
                .hover(|this| {
                    this.bg(rgb(0x063544))
                        .border_color(rgb(0x00e5ff))
                        .text_color(rgb(0x38f8ff))
                })
                .child(div().size(px(14.0)).text_color(rgb(0x00f0ff)).child(PandoraIcon::Plus))
                .child("Add Account")
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.is_account_flyout_open = false;
                    crate::modals::accounts::open_add_account_dialog(&backend_handle, window, cx);
                }))
        };

        let mut account_items = Vec::new();
        for (idx, account) in accounts.iter().enumerate() {
            let is_selected = Some(account.uuid) == selected_uuid;
            let account_uuid = account.uuid;
            let account_name = account.username(hide_usernames);
            let badge_text = if account.is_elyby {
                "ELY.BY"
            } else if account.offline {
                "OFFLINE"
            } else {
                "MICROSOFT"
            };

            let head_img = if hide_skins {
                gpui::img(ImageSource::Resource(Resource::Embedded("images/hidden_head.png".into())))
            } else if let Some(head) = &account.head {
                let resize = png_render_cache::ImageTransformation::Resize { width: 30, height: 30 };
                png_render_cache::render_with_transform(head.clone(), resize, cx)
            } else {
                gpui::img(ImageSource::Resource(Resource::Embedded("images/default_head.png".into())))
            };

            let row_group: SharedString = format!("flyout-account-row-{}", idx).into();
            let bh = backend_handle.clone();
            let bh_del = backend_handle.clone();

            let row = h_flex()
                .id(("flyout-account", idx))
                .group(row_group.clone())
                .relative()
                .w_full()
                .h(px(38.0))
                .px_2()
                .gap_2()
                .items_center()
                .rounded_none()
                .cursor_pointer()
                .when_else(is_selected, |this| {
                    this.bg(rgb(0x04242e))
                        .border_1()
                        .border_color(rgb(0x00e5ff))
                }, |this| {
                    this.bg(gpui::transparent_black())
                        .border_1()
                        .border_color(gpui::transparent_black())
                        .hover(|this| {
                            this.bg(rgba(0xffffff0d))
                                .border_color(rgb(0x00e5ff).opacity(0.35))
                        })
                })
                .on_click(cx.listener(move |this, _, _, cx| {
                    if !is_selected {
                        bh.send(MessageToBackend::SelectAccount { uuid: account_uuid });
                    }
                    this.is_account_flyout_open = false;
                    cx.notify();
                }))
                .child(
                    div()
                        .size(px(30.0))
                        .min_w(px(30.0))
                        .min_h(px(30.0))
                        .rounded_none()
                        .overflow_hidden()
                        .flex_none()
                        .child(head_img.size(px(30.0)).min_w(px(30.0)).min_h(px(30.0)))
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .font_family(crate::interface_config::DEFAULT_FONT)
                        .text_size(px(12.0))
                        .text_color(if is_selected { rgb(0x00f0ff) } else { rgb(0xffffff) })
                        .truncate()
                        .child(account_name)
                )
                .child(
                    div()
                        .text_size(rems(0.6875))
                        .px_1p5()
                        .py_0p5()
                        .rounded_none()
                        .bg(rgb(0x04242e))
                        .border_1()
                        .border_color(rgb(0x00e5ff).opacity(0.4))
                        .text_color(rgb(0x00f0ff))
                        .flex_none()
                        .child(badge_text)
                )
                .child(
                    Button::new(("del-account", idx))
                        .invisible()
                        .group_hover(row_group.clone(), |style| style.visible())
                        .ghost()
                        .compact()
                        .icon(PandoraIcon::Trash2)
                        .h(px(22.0))
                        .w(px(22.0))
                        .flex_none()
                        .danger()
                        .on_click(move |_, _, cx| {
                            cx.stop_propagation();
                            bh_del.send(MessageToBackend::DeleteAccount { uuid: account_uuid });
                        })
                );

            account_items.push(row.into_any_element());
        }

        let flyout_panel = v_flex()
            .id("account-flyout-panel")
            .occlude()
            .w_full()
            .p_1p5()
            .gap_1p5()
            .bg(rgb(0x040e0e))
            .border_1()
            .border_color(rgb(0x00e5ff))
            .rounded_none()
            .shadow_2xl()
            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                window.prevent_default();
                cx.stop_propagation();
            })
            .on_click(|_, _, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                this.is_account_flyout_open = false;
                this.account_flyout_closed_at = Some(std::time::Instant::now());
                cx.notify();
            }))
            .child(add_btn)
            .when(!account_items.is_empty(), |this| {
                this.child(
                    v_flex()
                        .w_full()
                        .gap_1()
                        .max_h(px(240.0))
                        .overflow_y_scrollbar()
                        .children(account_items)
                )
            });

        let flyout_container = div()
            .id("account-flyout-container")
            .occlude()
            .absolute()
            .bottom(px(56.0))
            .left(px(8.0))
            .w(px(270.0));

        flyout_container.child(flyout_panel).into_any_element()
    }
}

impl Render for LauncherUI {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(pending_page) = self.pending_page.clone() {
            if let Ok(page) = Self::create_page(&self.data, pending_page.0.clone(), window, cx) {
                self.pending_page = None;
                self.previous_pages.clear();
                self.page_history_forwards.clear();
                self.page_history_backwards.clear();

                let config = InterfaceConfig::get_mut(cx);
                config.main_page = pending_page.0.clone();
                config.page_path = pending_page.1.clone();

                self.page = page;
            }
        }

        let sidebar_width = {
            let width = InterfaceConfig::get(cx).sidebar_width;
            if width <= 0.0 {
                DEFAULT_SIDEBAR_WIDTH
            } else {
                width
            }
        };
        let iconized_sidebar = sidebar_width == ICONIZED_SIDEBAR_WIDTH;
        let page_type = InterfaceConfig::get(cx).main_page.clone();

        let page_groups: &[(&'static str, &[(bool, &'static str, PandoraIcon, PageType)])] = &[
            ("Library", &[
                (true, t::instance::title(), PandoraIcon::Play, PageType::Instances),
                (true, "Discover Content", PandoraIcon::Compass, PageType::Modrinth { installing_for: None }),
            ]),
        ];

        let mut instances: Vec<InstanceEntry> = self.data
            .instances
            .read(cx)
            .entries
            .values()
            .filter_map(|entry| {
                let entry = entry.read(cx).clone();
                if entry.name == schema::quickplay::INSTANCE_NAME {
                    None
                } else {
                    Some(entry)
                }
            })
            .collect();

        let saved_order = &InterfaceConfig::get(cx).sidebar_instance_order;
        if !saved_order.is_empty() {
            instances.sort_by_key(|inst| {
                saved_order.iter().position(|name| &**name == inst.name.as_str()).unwrap_or(usize::MAX)
            });
        }

        if !cx.has_active_drag() {
            let mut reordered = None;
            self.sidebar_reorderable_state.update(cx, |state, _| {
                if let Some((from, to)) = state.take_reorder() {
                    reordered = Some((from, to));
                }
            });
            if let Some((from, to)) = reordered {
                if from < instances.len() && to < instances.len() {
                    let moved = instances.remove(from);
                    instances.insert(to, moved);
                    let order = &mut InterfaceConfig::get_mut(cx).sidebar_instance_order;
                    *order = instances.iter().map(|inst| Arc::from(inst.name.as_str())).collect();
                }
            }
        }

        let sidebar_pages = if !iconized_sidebar {
            let mut groups: Vec<AnyElement> = Vec::new();

            for (group_name, group_entries) in page_groups {
                let mut group_items: Vec<AnyElement> = Vec::new();
                for (show, title, icon, page) in *group_entries {
                    if !*show {
                        continue;
                    }
                    let title = *title;
                    let page = page.clone();
                    let is_active = match (&page_type, &page) {
                        (PageType::Modrinth { .. } | PageType::Curseforge { .. } | PageType::ModrinthProject { .. }, PageType::Modrinth { .. }) => true,
                        (a, b) => a == b,
                    };
                    group_items.push(
                        h_flex()
                            .id(title)
                            .w_full()
                            .h(px(36.0))
                            .px_3()
                            .gap_2p5()
                            .items_center()
                            .rounded_none()
                            .cursor_pointer()
                            .when_else(is_active, |this| {
                                this.bg(rgba(0x00e5ff1a))
                                    .border_1()
                                    .border_color(rgb(0x00e5ff))
                                    .text_color(rgb(0xffffff))
                            }, |this| {
                                this.text_color(rgb(0x94a3b8))
                                    .hover(|this| {
                                        this.bg(rgba(0xffffff0d))
                                            .text_color(rgb(0xffffff))
                                    })
                            })
                            .child(
                                div()
                                    .size(px(18.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .flex_none()
                                    .when(is_active, |this| this.text_color(rgb(0x00e5ff)))
                                    .child(icon.clone())
                            )
                            .child(
                                div().text_sm().font_medium().child(title)
                            )
                            .on_click(cx.listener(move |launcher, _, window, cx| {
                                launcher.switch_page(page.clone(), &[], window, cx);
                            }))
                            .into_any_element()
                    );
                }

                groups.push(
                    v_flex()
                        .w_full()
                        .gap_y_1()
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_bold()
                                .text_color(cx.theme().muted_foreground)
                                .px_2()
                                .pb_1()
                                .child(group_name.to_uppercase())
                        )
                        .children(group_items)
                        .into_any_element()
                );
            }

            if !instances.is_empty() {
                let instances_cloned = instances.clone();
                let current_page = page_type.clone();
                let tasks_cloned = self.data.tasks.clone();
                let reorderable = Reorderable::new(&self.sidebar_reorderable_state, instances.len(), cx, move |render_index, _info, _, cx| {
                    let Some(instance) = instances_cloned.get(render_index) else {
                        return div().into_any_element();
                    };
                    let name = instance.name.clone();
                    let active = current_page == PageType::InstancePage { name: name.clone() };
                    let active_task = tasks_cloned.read(cx).snapshots().into_iter().find(|t| !t.is_finished && (t.id == Some(instance.id) || t.name == instance.name));
                    let download_pct = active_task.as_ref().map(|t| {
                        if t.total > 0 {
                            (t.count as f32 / t.total as f32).clamp(0.01, 1.0)
                        } else {
                            0.08
                        }
                    });
                    let tile = crate::component::instance_icon::render_sidebar_tile_with_download(
                        instance.icon.clone(),
                        &instance.name,
                        instance.configuration.instance_fallback_icon.as_deref(),
                        download_pct,
                        cx,
                    );

                    h_flex()
                        .id(name.clone())
                        .w_full()
                        .p_1p5()
                        .gap_2()
                        .items_center()
                        .rounded_none()
                        .cursor_pointer()
                        .when_else(active, |this| {
                            this.bg(rgba(0x00f0ff14))
                                .border_1()
                                .border_color(rgb(0x00f0ff))
                                .text_color(rgb(0xffffff))
                        }, |this| {
                            this.border_1()
                                .border_color(gpui::transparent_black())
                                .text_color(rgb(0x94a3b8))
                                .hover(|this| {
                                    this.bg(rgba(0xffffff0a))
                                        .text_color(rgb(0xffffff))
                                })
                        })
                        .child(tile)
                        .child(
                            div()
                                .text_sm()
                                .font_medium()
                                .truncate()
                                .flex_1()
                                .child(instance.name.clone())
                        )
                        .on_click(move |_, window, cx| {
                            crate::root::switch_page(PageType::InstancePage { name: name.clone() }, &[PageType::Instances], window, cx);
                        })
                        .into_any_element()
                }).w_full().v_flex().gap_1();

                groups.push(
                    v_flex()
                        .w_full()
                        .gap_y_1()
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_bold()
                                .text_color(cx.theme().muted_foreground)
                                .px_2()
                                .pb_1()
                                .child("INSTANCES")
                        )
                        .child(reorderable)
                        .into_any_element()
                );
            }

            v_flex()
                .flex_1()
                .min_h_0()
                .px_3()
                .gap_y_3()
                .children(groups)
                .overflow_y_scrollbar()
                .into_any_element()
        } else {
            let mut groups = Vec::new();

            for (_group_name, group_entries) in page_groups {
                let mut group = v_flex()
                    .items_center().min_w_full().max_w_full().w_full()
                    .min_h_0()
                    .gap_y_1();
                for (show, title, icon, page) in *group_entries {
                    if !*show {
                        continue;
                    }
                    let title = *title;
                    let page = page.clone();
                    let is_active = match (&page_type, &page) {
                        (PageType::Modrinth { .. } | PageType::Curseforge { .. } | PageType::ModrinthProject { .. }, PageType::Modrinth { .. }) => true,
                        (a, b) => a == b,
                    };
                    group = group.child(
                        div()
                            .id(title)
                            .size(px(36.0))
                            .rounded_none()
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .justify_center()
                            .when_else(is_active, |this| {
                                this.bg(rgba(0x00e5ff1a))
                                    .border_1()
                                    .border_color(rgb(0x00e5ff))
                                    .text_color(rgb(0x00e5ff))
                            }, |this| {
                                this.text_color(rgb(0x94a3b8))
                                    .hover(|this| {
                                        this.bg(rgba(0xffffff0d))
                                            .text_color(rgb(0xffffff))
                                    })
                            })
                            .child(icon.clone())
                            .tooltip(move |window, cx| {
                                Tooltip::new(title).build(window, cx)
                            })
                            .on_click(cx.listener(move |launcher, _, window, cx| {
                                launcher.switch_page(page.clone(), &[], window, cx);
                            }))
                    );
                }
                groups.push(group.into_any_element());
            }

            if !instances.is_empty() {
                let instances_cloned = instances.clone();
                let current_page = page_type.clone();
                let tasks_cloned = self.data.tasks.clone();
                let reorderable = Reorderable::new(&self.sidebar_reorderable_state, instances.len(), cx, move |render_index, _info, _, cx| {
                    let Some(instance) = instances_cloned.get(render_index) else {
                        return div().into_any_element();
                    };
                    let name = instance.name.clone();
                    let active = current_page == PageType::InstancePage { name: name.clone() };
                    let active_task = tasks_cloned.read(cx).snapshots().into_iter().find(|t| !t.is_finished && (t.id == Some(instance.id) || t.name == instance.name));
                    let download_pct = active_task.as_ref().map(|t| {
                        if t.total > 0 {
                            (t.count as f32 / t.total as f32).clamp(0.01, 1.0)
                        } else {
                            0.08
                        }
                    });
                    let tile = crate::component::instance_icon::render_sidebar_tile_with_download(
                        instance.icon.clone(),
                        &instance.name,
                        instance.configuration.instance_fallback_icon.as_deref(),
                        download_pct,
                        cx,
                    );

                    div()
                        .id(name.clone())
                        .p_1()
                        .rounded_none()
                        .cursor_pointer()
                        .when_else(active, |this| {
                            this.bg(cx.theme().sidebar_accent)
                                .text_color(cx.theme().sidebar_accent_foreground)
                        }, |this| {
                            this.hover(|this| {
                                this.bg(cx.theme().sidebar_accent.opacity(0.8))
                                    .text_color(cx.theme().sidebar_accent_foreground)
                            })
                        })
                        .child(tile)
                        .tooltip({
                            let name = name.clone();
                            move |window, cx| {
                                Tooltip::new(name.clone()).build(window, cx)
                            }
                        })
                        .on_click(move |_, window, cx| {
                            crate::root::switch_page(PageType::InstancePage { name: name.clone() }, &[PageType::Instances], window, cx);
                        })
                        .into_any_element()
                }).w_full().v_flex().gap_1();

                groups.push(reorderable.into_any_element());
            }

            v_flex()
                .items_center().min_w_full().max_w_full().w_full()
                .min_h_0()
                .pt_1()
                .px_2()
                .gap_y_3()
                .children(groups)
                .overflow_y_scrollbar()
                .into_any_element()
        };


        let selected_account_data = {
            let accounts = self.data.accounts.read(cx);
            accounts.selected_account.as_ref().map(|account| {
                (
                    account.username(InterfaceConfig::get(cx).hide_usernames),
                    account.head.clone(),
                    if account.is_elyby {
                        Some("Ely.by")
                    } else if account.offline {
                        Some("Offline")
                    } else {
                        Some("Microsoft")
                    },
                )
            })
        };

        let (account_head, account_name, account_badge) = if let Some((account_name, head_data, badge)) = selected_account_data {
            let hide_skins = InterfaceConfig::get(cx).hide_skins;

            let head = if hide_skins {
                gpui::img(ImageSource::Resource(Resource::Embedded("images/hidden_head.png".into())))
            } else if let Some(head) = head_data {
                let resize = png_render_cache::ImageTransformation::Resize { width: 34, height: 34 };
                png_render_cache::render_with_transform(head, resize, cx)
            } else {
                gpui::img(ImageSource::Resource(Resource::Embedded("images/default_head.png".into())))
            };
            (head, account_name, badge)
        } else {
            (
                gpui::img(ImageSource::Resource(Resource::Embedded("images/default_head.png".into()))),
                t::account::none().into(),
                None,
            )
        };

        let account_tile = div()
            .size(px(34.0))
            .min_size(px(34.0))
            .rounded_none()
            .overflow_hidden()
            .flex_none()
            .child(account_head.size(px(34.0)).min_w(px(34.0)).min_h(px(34.0)));

        let is_flyout_open = self.is_account_flyout_open;

        let account_button = if !iconized_sidebar {
            h_flex()
                .id("account-button")
                .w_full()
                .p_1p5()
                .gap_3()
                .items_center()
                .rounded_none()
                .cursor_pointer()
                .when_else(is_flyout_open, |this| {
                    this.border_1()
                        .border_color(rgb(0x00e5ff))
                        .bg(rgb(0x04242e))
                }, |this| {
                    this.border_1()
                        .border_color(gpui::transparent_black())
                        .hover(|this| {
                            this.bg(cx.theme().sidebar_accent)
                                .text_color(cx.theme().sidebar_accent_foreground)
                        })
                })
                .child(account_tile)
                .child(
                    h_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_family(crate::interface_config::DEFAULT_FONT)
                                .text_size(px(12.0))
                                .truncate()
                                .child(account_name)
                        )
                        .when_some(account_badge, |this, badge| {
                            this.child(
                                div()
                                    .text_size(rems(0.6875))
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_none()
                                    .bg(rgb(0x04242e))
                                    .border_1()
                                    .border_color(rgb(0x00e5ff).opacity(0.4))
                                    .text_color(rgb(0x00f0ff))
                                    .child(badge)
                            )
                        })
                )
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(closed_at) = this.account_flyout_closed_at.take() {
                        if closed_at.elapsed().as_millis() < 300 {
                            return;
                        }
                    }
                    this.is_account_flyout_open = !this.is_account_flyout_open;
                    cx.notify();
                }))
                .into_any_element()
        } else {
            div()
                .id("account-button")
                .p_1()
                .rounded_none()
                .cursor_pointer()
                .when_else(is_flyout_open, |this| {
                    this.border_1()
                        .border_color(rgb(0x00e5ff))
                        .bg(rgb(0x04242e))
                }, |this| {
                    this.border_1()
                        .border_color(gpui::transparent_black())
                        .hover(|this| {
                            this.bg(cx.theme().sidebar_accent)
                                .text_color(cx.theme().sidebar_accent_foreground)
                        })
                })
                .child(account_tile)
                .tooltip(move |window, cx| {
                    Tooltip::new(account_name.clone()).build(window, cx)
                })
                .on_click(cx.listener(|this, _, _, cx| {
                    if let Some(closed_at) = this.account_flyout_closed_at.take() {
                        if closed_at.elapsed().as_millis() < 300 {
                            return;
                        }
                    }
                    this.is_account_flyout_open = !this.is_account_flyout_open;
                    cx.notify();
                }))
                .into_any_element()
        };

        let settings_button = if !iconized_sidebar {
            h_flex()
                .id("settings-button")
                .w_full()
                .h(px(36.0))
                .px_3()
                .gap_2p5()
                .items_center()
                .rounded_none()
                .cursor_pointer()
                .when_else(page_type == PageType::Settings, |this| {
                    this.bg(rgba(0x00e5ff1a))
                        .border_1()
                        .border_color(rgb(0x00e5ff))
                        .text_color(rgb(0xffffff))
                }, |this| {
                    this.text_color(rgb(0x94a3b8))
                        .hover(|this| {
                            this.bg(rgba(0xffffff0d))
                                .text_color(rgb(0xffffff))
                        })
                })
                .child(
                    div()
                        .size(px(18.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_none()
                        .when(page_type == PageType::Settings, |this| this.text_color(rgb(0x00e5ff)))
                        .child(PandoraIcon::Settings)
                )
                .child(div().text_sm().font_medium().child(t::settings::title()))
                .on_click(cx.listener(|this, _, window, cx| {
                    crate::root::open_settings_modal(&this.data, window, cx);
                }))
                .into_any_element()
        } else {
            div()
                .id("settings-button")
                .size(px(36.0))
                .rounded_none()
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .when_else(page_type == PageType::Settings, |this| {
                    this.bg(rgba(0x00e5ff1a))
                        .border_1()
                        .border_color(rgb(0x00e5ff))
                        .text_color(rgb(0x00e5ff))
                }, |this| {
                    this.text_color(rgb(0x94a3b8))
                        .hover(|this| {
                            this.bg(rgba(0xffffff0d))
                                .text_color(rgb(0xffffff))
                        })
                })
                .child(PandoraIcon::Settings)
                .tooltip(move |window, cx| {
                    Tooltip::new(t::settings::title()).build(window, cx)
                })
                .on_click(cx.listener(|this, _, window, cx| {
                    crate::root::open_settings_modal(&this.data, window, cx);
                }))
                .into_any_element()
        };

        let wardrobe_button = if !iconized_sidebar {
            h_flex()
                .id("wardrobe-button")
                .w_full()
                .h(px(36.0))
                .px_3()
                .gap_2p5()
                .items_center()
                .rounded_none()
                .cursor_pointer()
                .when_else(page_type == PageType::Skins, |this| {
                    this.bg(rgba(0x00e5ff1a))
                        .border_1()
                        .border_color(rgb(0x00e5ff))
                        .text_color(rgb(0xffffff))
                }, |this| {
                    this.text_color(rgb(0x94a3b8))
                        .hover(|this| {
                            this.bg(rgba(0xffffff0d))
                                .text_color(rgb(0xffffff))
                        })
                })
                .child(
                    div()
                        .size(px(18.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_none()
                        .when(page_type == PageType::Skins, |this| this.text_color(rgb(0x00e5ff)))
                        .child(PandoraIcon::Hanger)
                )
                .child(div().text_sm().font_medium().child("Wardrobe"))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.switch_page(PageType::Skins, &[], window, cx);
                }))
                .into_any_element()
        } else {
            div()
                .id("wardrobe-button")
                .size(px(36.0))
                .rounded_none()
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .when_else(page_type == PageType::Skins, |this| {
                    this.bg(rgba(0x00e5ff1a))
                        .border_1()
                        .border_color(rgb(0x00e5ff))
                        .text_color(rgb(0x00e5ff))
                }, |this| {
                    this.text_color(rgb(0x94a3b8))
                        .hover(|this| {
                            this.bg(rgba(0xffffff0d))
                                .text_color(rgb(0xffffff))
                        })
                })
                .child(PandoraIcon::Hanger)
                .tooltip(move |window, cx| {
                    Tooltip::new("Wardrobe").build(window, cx)
                })
                .on_click(cx.listener(|this, _, window, cx| {
                    this.switch_page(PageType::Skins, &[], window, cx);
                }))
                .into_any_element()
        };

        let header_drag_state = window.use_keyed_state("sidebar-header-drag-state", cx, |_, _| TitleBarState::default());
        let header = h_flex()
            .id("sidebar-header")
            .window_control_area(WindowControlArea::Drag)
            .on_mouse_down_out(window.listener_for(&header_drag_state, |state, _, _, _| {
                state.should_move = false;
            }))
            .when(cfg!(target_os = "linux"), |this| {
                this.on_double_click(|_, window, _| window.zoom_window())
            })
            .when(cfg!(target_os = "macos"), |this| {
                this.on_double_click(|_, window, _| window.titlebar_double_click())
            })
            .on_mouse_down(
                MouseButton::Left,
                window.listener_for(&header_drag_state, |state, _, _, _| {
                    state.should_move = true;
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                window.listener_for(&header_drag_state, |state, _, _, _| {
                    state.should_move = false;
                }),
            )
            .on_mouse_move(window.listener_for(&header_drag_state, |state, _, window, _| {
                if state.should_move {
                    state.should_move = false;
                    window.start_window_move();
                }
            }))
            .when_else(cfg!(target_os = "macos"), |this| this.pt(px(41.0)), |this| this.pt(px(14.0)))
            .px_5()
            .pb_2()
            .gap_2()
            .w_full()
            .justify_center()
            .text_size(rems(0.9375))
            .child(gpui::img(ImageSource::Resource(Resource::Embedded("images/logo.png".into()))).size_8().min_w_8().min_h_8())
            .when(!iconized_sidebar, |this| this.child(t::common::app_name()));
        let footer = v_flex()
            .pb_2()
            .px_2()
            .gap_y_1()
            .items_center()
            .min_w_full()
            .max_w_full()
            .w_full()
            .relative()
            .child(settings_button)
            .child(wardrobe_button)
            .child(account_button)
            .when(self.is_account_flyout_open, |this| {
                this.child(self.render_account_flyout(iconized_sidebar, window, cx))
            });
        let sidebar = v_flex()
            .w(px(sidebar_width))
            .min_w(px(sidebar_width))
            .max_w(px(sidebar_width))
            .h_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .border_r_1()
            .border_color(cx.theme().border)
            .child(header)
            .child(sidebar_pages)
            .child(footer);

        let current_page = InterfaceConfig::get(cx).main_page.clone();
        let page_anim_id = format!("main_page_{:?}", current_page);

        h_flex()
            .size_full()
            .child(sidebar)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .relative()
                    .with_animation(
                        page_anim_id,
                        Animation::new(std::time::Duration::from_millis(220))
                            .with_easing(ease_out_quint()),
                        |div, delta| {
                            div.opacity(delta).top(px((1.0 - delta) * 8.0))
                        },
                    )
                    .child(self.page.clone().render(&self, window, cx))
            )
    }
}

pub fn open_bug_report_url(window: &mut Window, cx: &mut App) {
    let mut body = String::from(r#"## Description of bug
(Write here)

## Steps to reproduce
(Write here)

## This issue is unique
- [ ] I've searched the other issues and didn't see an issue describing the same bug

## Environment
"#);

    use std::fmt::Write;
    _ = writeln!(&mut body, "Version: {:?}", option_env!("PANDORA_RELEASE_VERSION"));
    _ = writeln!(&mut body, "Distributor: {:?}", option_env!("PANDORA_DISTRIBUTION"));
    _ = writeln!(&mut body, "OS: {} ({})", std::env::consts::OS, std::env::consts::ARCH);

    if cfg!(target_os = "linux") {
        if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
            for line in os_release.lines() {
                let line = line.trim_ascii();
                if let Some(name) = line.strip_prefix("NAME=") {
                    _ = writeln!(&mut body, "OS Name: {}", name);
                } else if let Some(version) = line.strip_prefix("VERSION=") {
                    _ = writeln!(&mut body, "OS Version: {}", version);
                }
            }
        }

        _ = writeln!(&mut body, "Desktop: {:?}", std::env::var_os("XDG_CURRENT_DESKTOP"));

        if let Some(snap_name) = std::env::var_os("SNAP_NAME") {
            _ = writeln!(&mut body, "Snap: {:?}", snap_name);
        }
        if let Some(snap_name) = std::env::var_os("FLATPAK_ID") {
            _ = writeln!(&mut body, "Flatpak ID: {:?}", snap_name);
        }
        if std::env::var_os("APPIMAGE").is_some() {
            body.push_str("AppImage: true\n");
        }
    }

    let Some(github) = option_env!("GITHUB_REPOSITORY_URL") else {
        let mut notification: Notification = (NotificationType::Error, SharedString::from("Unable to report bug, GITHUB_REPOSITORY_URL was not set at compile time")).into();
        notification = notification.autohide(false);
        window.push_notification(notification, cx);
        return;
    };

    cx.open_url(&format!("{}/issues/new?body={}", github, urlencoding::encode(&body)));
}

pub struct SidebarDragPreview {
    pub name: SharedString,
    pub icon: Option<schema::unique_bytes::UniqueBytes>,
    pub fallback_icon: Option<ustr::Ustr>,
}

impl Render for SidebarDragPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tile = crate::component::instance_icon::render_sidebar_tile(
            self.icon.clone(),
            &self.name,
            self.fallback_icon.as_deref(),
            cx,
        );

        div()
            .absolute()
            .child(
                h_flex()
                    .left(px(-185.0))
                    .top(px(-18.0))
                    .w(px(210.0))
                    .p_1p5()
                    .gap_2()
                    .items_center()
                    .rounded_none()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().popover)
                    .text_color(cx.theme().popover_foreground)
                    .shadow_xl()
                    .opacity(0.96)
                    .child(tile)
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .truncate()
                            .flex_1()
                            .child(self.name.clone())
                    )
            )
    }
}
