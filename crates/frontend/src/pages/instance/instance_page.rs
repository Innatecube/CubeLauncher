use bridge::{
    handle::BackendHandle,
    instance::{InstanceID, InstanceStatus},
    message::MessageToBackend,
};
use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme as _, Disableable, StyledExt as _, WindowExt, button::{Button, ButtonCustomVariant, ButtonGroup, ButtonVariants, DropdownButton}, h_flex, menu::PopupMenuItem, spinner::Spinner, tab::{Tab, TabBar}, v_flex
};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{DataEntities, instance::InstanceEntry}, game_output::GameOutputRoot, icon::PandoraIcon, interface_config::InterfaceConfig, pages::{instance::{content_subpage::InstanceContentSubpage, logs_subpage::InstanceLogsSubpage, settings_subpage::InstanceSettingsSubpage}, page::Page}, png_render_cache, root
};

use super::content_subpage::ContentType;

pub struct InstancePage {
    data: DataEntities,
    pub instance: Entity<InstanceEntry>,
    subpage: InstanceSubpage,
}

impl InstancePage {
    pub fn new(instance_id: InstanceID, data: &DataEntities, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let instance = data.instances.read(cx).entries.get(&instance_id).unwrap().clone();

        let mut instance_subpage = InterfaceConfig::get(cx).instance_subpage;
        if instance_subpage == InstanceSubpageType::Settings {
            InterfaceConfig::get_mut(cx).instance_subpage = InstanceSubpageType::Mods;
            instance_subpage = InstanceSubpageType::Mods;
        }
        let subpage = instance_subpage.create(&instance, data, data.backend_handle.clone(), window, cx);

        let subpage = subpage.unwrap_or_else(|| {
            InterfaceConfig::get_mut(cx).instance_subpage = InstanceSubpageType::Mods;
            InstanceSubpageType::Mods.create(&instance, data, data.backend_handle.clone(), window, cx).unwrap()
        });

        Self {
            data: data.clone(),
            instance,
            subpage,
        }
    }
}

impl Page for InstancePage {
    fn controls(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }

    fn scrollable(&self, _cx: &App) -> bool {
        false
    }
}

impl Render for InstancePage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut instance_subpage = InterfaceConfig::get(cx).instance_subpage;
        if instance_subpage == InstanceSubpageType::Settings {
            InterfaceConfig::get_mut(cx).instance_subpage = InstanceSubpageType::Mods;
            instance_subpage = InstanceSubpageType::Mods;
        }
        if instance_subpage != self.subpage.page_type() {
            let subpage = instance_subpage.create(&self.instance, &self.data, self.data.backend_handle.clone(), window, cx);

            self.subpage = subpage.unwrap_or_else(|| {
                InterfaceConfig::get_mut(cx).instance_subpage = InstanceSubpageType::Mods;
                InstanceSubpageType::Mods.create(&self.instance, &self.data, self.data.backend_handle.clone(), window, cx).unwrap()
            });
        }

        let (id, instance_name, status, icon, fallback_icon, loader_str, mc_ver, show_shader_tab, show_live_game_output) = {
            let entry = self.instance.read(cx);
            (
                entry.id,
                entry.name.clone(),
                entry.status,
                entry.icon.clone(),
                entry.configuration.instance_fallback_icon.clone(),
                entry.configuration.loader.pretty_name(),
                entry.configuration.minecraft_version.clone(),
                entry.configuration.show_shader_tab || matches!(self.subpage, InstanceSubpage::Shaders(_)),
                entry.live_game_output.clone(),
            )
        };
        let show_live_game_output_bool = show_live_game_output.is_some();

        // Update live game output
        if let InstanceSubpage::LiveGameOutput(current_output) = &self.subpage
            && let Some(desired_output) = &show_live_game_output
            && current_output != desired_output
        {
            self.subpage = InstanceSubpage::LiveGameOutput(desired_output.clone());
        }

        let selected_index = match &self.subpage {
            InstanceSubpage::Mods(_) => 0,
            InstanceSubpage::Logs(_) => 1,
            InstanceSubpage::ResourcePacks(_) => 2,
            InstanceSubpage::Shaders(_) => 3,
            InstanceSubpage::Settings(_) => 0,
            InstanceSubpage::LiveGameOutput(_) => if show_shader_tab { 4 } else { 3 },
        };

        let active_task = self.data.tasks.read(cx).snapshots().into_iter().find(|t| !t.is_finished && (t.id == Some(id) || t.name == instance_name));
        let download_pct = active_task.as_ref().map(|t| {
            if t.total > 0 {
                (t.count as f32 / t.total as f32).clamp(0.01, 1.0)
            } else {
                0.08
            }
        });

        let icon_element = crate::component::instance_icon::render_instance_tile_with_download(
            icon,
            &instance_name,
            fallback_icon.as_deref(),
            48.0,
            10.0,
            download_pct,
            cx,
        );

        let name = instance_name.clone();
        let data = self.data.clone();

        let custom_play = ButtonCustomVariant::new(cx)
            .color(rgb(0x04242e).into())
            .foreground(rgb(0x00f0ff).into())
            .hover(rgb(0x063e4d).into())
            .active(rgb(0x031c24).into());

        let is_installing = self.data.tasks.read(cx).is_active(id) || download_pct.is_some();
        let play_button = if is_installing {
            let label = if let Some(pct) = download_pct {
                format!("Installing... {}%", (pct * 100.0) as u32)
            } else {
                "Installing...".to_string()
            };
            Button::new("installing_instance")
                .custom(custom_play)
                .border_1()
                .border_color(rgb(0x00e5ff))
                .icon(Spinner::new())
                .label(label)
                .loading(true)
                .into_any_element()
        } else {
            match status {
                InstanceStatus::NotRunning => {
                    Button::new("start_instance")
                        .custom(custom_play)
                        .border_1()
                        .border_color(rgb(0x00e5ff))
                        .icon(PandoraIcon::Play)
                        .label("Play")
                        .on_click(
                            move |_, window, cx| {
                                root::start_instance(id, name.clone(), None, &data, window, cx);
                            },
                        ).into_any_element()
                },
            InstanceStatus::Launching => {
                let name = name.clone();
                let data = data.clone();
                DropdownButton::new("launching_split")
                    .warning()
                    .rounded_none()
                    .button(
                        Button::new("launching")
                            .warning()
                            .icon(Spinner::new())
                            .label(t::instance::start::starting())
                            .loading(true)
                    )
                    .dropdown_menu({
                        let data = data.clone();
                        let name = name.clone();
                        move |this, _, _| {
                            let restart_item = PopupMenuItem::new("Restart Instance")
                                .icon(PandoraIcon::RefreshCcw)
                                .on_click({
                                    let data = data.clone();
                                    let name = name.clone();
                                    move |_, window, cx| {
                                        crate::root::restart_instance(id, name.clone(), None, &data, window, cx);
                                    }
                                });
                            let stop_item = PopupMenuItem::new(t::instance::kill_instance())
                                .icon(PandoraIcon::Close)
                                .on_click({
                                    let backend_handle = data.backend_handle.clone();
                                    move |_, _, _| {
                                        backend_handle.send(MessageToBackend::KillInstance { id });
                                    }
                                });
                            this.item(restart_item).item(stop_item)
                        }
                    })
                    .into_any_element()
            },
            InstanceStatus::Stopping => {
                Button::new("stopping")
                    .danger()
                    .icon(Spinner::new())
                    .label(t::instance::start::stopping())
                    .loading(true)
                    .on_click({
                        let backend_handle = data.backend_handle.clone();
                        move |_, _, _| {
                            backend_handle.send(MessageToBackend::KillInstance { id });
                        }
                    })
                    .into_any_element()
            },
            InstanceStatus::Running => {
                let name = name.clone();
                let data = data.clone();
                DropdownButton::new("running_split")
                    .danger()
                    .rounded_none()
                    .button(
                        Button::new("kill_instance")
                            .danger()
                            .icon(PandoraIcon::Close)
                            .label(t::instance::kill_instance())
                            .on_click({
                                let backend_handle = data.backend_handle.clone();
                                move |_, _, _| {
                                    backend_handle.send(MessageToBackend::KillInstance { id });
                                }
                            })
                    )
                    .dropdown_menu({
                        let data = data.clone();
                        let name = name.clone();
                        move |this, _, _| {
                            let stop_item = PopupMenuItem::new(t::instance::kill_instance())
                                .icon(PandoraIcon::Close)
                                .on_click({
                                    let backend_handle = data.backend_handle.clone();
                                    move |_, _, _| {
                                        backend_handle.send(MessageToBackend::KillInstance { id });
                                    }
                                });
                            let restart_item = PopupMenuItem::new("Restart Instance")
                                .icon(PandoraIcon::RefreshCcw)
                                .on_click({
                                    let data = data.clone();
                                    let name = name.clone();
                                    move |_, window, cx| {
                                        crate::root::restart_instance(id, name.clone(), None, &data, window, cx);
                                    }
                                });
                            this.item(stop_item).item(restart_item)
                        }
                    })
                    .into_any_element()
            },
        }
    };

        let instance_for_settings = self.instance.clone();
        let data_for_settings = self.data.clone();
        let settings_button = Button::new("instance_settings")
            .ghost()
            .icon(PandoraIcon::Settings)
            .tooltip(t::settings::title())
            .on_click(move |_, window, cx| {
                crate::pages::instance::settings_subpage::open_instance_settings_modal(
                    &instance_for_settings,
                    &data_for_settings,
                    window,
                    cx,
                );
            });

        let dot_minecraft = self.instance.read(cx).dot_minecraft_folder.clone();
        let open_dot_minecraft_button = Button::new("open_dot_minecraft")
            .ghost()
            .icon(PandoraIcon::FolderOpen)
            .tooltip(t::instance::open_folder())
            .on_click({
                let dot_minecraft = dot_minecraft.clone();
                move |_, window, cx| {
                    crate::open_folder(&dot_minecraft, window, cx);
                }
            });

        let instance_header = h_flex()
            .px_4()
            .pt_3()
            .pb_1()
            .gap_3()
            .items_center()
            .child(icon_element)
            .child(
                v_flex()
                    .gap_0p5()
                    .child(div().text_size(rems(1.25)).font_weight(FontWeight::BOLD).text_color(cx.theme().foreground).child(instance_name.clone()))
                    .child(
                        h_flex()
                            .gap_1p5()
                            .items_center()
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_none()
                                    .bg(rgb(0x111724))
                                    .border_1()
                                    .border_color(rgb(0x1e2a40))
                                    .text_size(px(11.0))
                                    .font_semibold()
                                    .text_color(rgb(0x00e5ff))
                                    .child(loader_str)
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_none()
                                    .bg(rgb(0x111724))
                                    .border_1()
                                    .border_color(rgb(0x1e2a40))
                                    .text_size(px(11.0))
                                    .font_medium()
                                    .text_color(rgb(0x94a3b8))
                                    .child(mc_ver.to_string())
                            )
                    )
            )
            .child(div().flex_1())
            .child(play_button)
            .child(settings_button)
            .child(open_dot_minecraft_button);



        v_flex()
            .size_full()
            .child(instance_header)
            .child(
                TabBar::new("bar")
                    .prefix(div().w_4())
                    .selected_index(selected_index)
                    .underline()
                    .child(Tab::new().label(t::instance::content::mods()))
                    .child(Tab::new().label(t::instance::logs::title()))
                    .child(Tab::new().label(t::instance::content::resourcepacks()))
                    .when(show_shader_tab, |this| {
                        this.child(Tab::new().label(t::instance::content::shaders()))
                    })
                    .when(show_live_game_output_bool, |this| {
                        this.child(Tab::new().label(t::instance::live_game_output()))
                    })
                    .on_click(cx.listener(move |_, index, _, cx| {
                        let page_type = match *index {
                            0 => InstanceSubpageType::Mods,
                            1 => InstanceSubpageType::Logs,
                            2 => InstanceSubpageType::ResourcePacks,
                            3 => if show_shader_tab {
                                InstanceSubpageType::Shaders
                            } else if show_live_game_output_bool {
                                InstanceSubpageType::LiveGameOutput
                            } else {
                                return;
                            },
                            4 => {
                                if show_shader_tab && show_live_game_output_bool {
                                    InstanceSubpageType::LiveGameOutput
                                } else {
                                    return;
                                }
                            },
                            _ => {
                                return;
                            },
                        };
                        InterfaceConfig::get_mut(cx).instance_subpage = page_type;
                    })),
            )
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .relative()
                    .with_animation(
                        ("instance_tab", selected_index),
                        Animation::new(std::time::Duration::from_millis(200))
                            .with_easing(ease_out_quint()),
                        |div, delta| {
                            div.opacity(delta).top(px((1.0 - delta) * 6.0))
                        },
                    )
                    .child(self.subpage.clone().into_any_element())
            )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceSubpageType {
    #[default]
    Mods,
    Logs,
    ResourcePacks,
    Shaders,
    Settings,
    LiveGameOutput,
}

impl InstanceSubpageType {
    pub fn create(
        self,
        instance: &Entity<InstanceEntry>,
        data: &DataEntities,
        backend_handle: BackendHandle,
        window: &mut gpui::Window,
        cx: &mut App
    ) -> Option<InstanceSubpage> {
        Some(match self {
            InstanceSubpageType::Mods => InstanceSubpage::Mods(cx.new(|cx| {
                InstanceContentSubpage::new(instance, ContentType::Mods, data, backend_handle, window, cx)
            })),
            InstanceSubpageType::Logs => InstanceSubpage::Logs(cx.new(|cx| {
                InstanceLogsSubpage::new(instance, backend_handle, window, cx)
            })),
            InstanceSubpageType::ResourcePacks => InstanceSubpage::ResourcePacks(cx.new(|cx| {
                InstanceContentSubpage::new(instance, ContentType::ResourcePacks, data, backend_handle, window, cx)
            })),
            InstanceSubpageType::Shaders => InstanceSubpage::Shaders(cx.new(|cx| {
                InstanceContentSubpage::new(instance, ContentType::Shaders, data, backend_handle, window, cx)
            })),
            InstanceSubpageType::Settings => InstanceSubpage::Mods(cx.new(|cx| {
                InstanceContentSubpage::new(instance, ContentType::Mods, data, backend_handle, window, cx)
            })),
            InstanceSubpageType::LiveGameOutput => {
                if let Some(game_output) = instance.read(cx).live_game_output.clone() {
                    InstanceSubpage::LiveGameOutput(game_output)
                } else {
                    return None;
                }
            },
        })
    }
}

#[derive(Clone)]
pub enum InstanceSubpage {
    Logs(Entity<InstanceLogsSubpage>),
    Mods(Entity<InstanceContentSubpage>),
    ResourcePacks(Entity<InstanceContentSubpage>),
    Shaders(Entity<InstanceContentSubpage>),
    Settings(Entity<InstanceSettingsSubpage>),
    LiveGameOutput(Entity<GameOutputRoot>),
}

impl InstanceSubpage {
    pub fn page_type(&self) -> InstanceSubpageType {
        match self {
            InstanceSubpage::Mods(_) => InstanceSubpageType::Mods,
            InstanceSubpage::Logs(_) => InstanceSubpageType::Logs,
            InstanceSubpage::ResourcePacks(_) => InstanceSubpageType::ResourcePacks,
            InstanceSubpage::Shaders(_) => InstanceSubpageType::Shaders,
            InstanceSubpage::Settings(_) => InstanceSubpageType::Settings,
            InstanceSubpage::LiveGameOutput(_) => InstanceSubpageType::LiveGameOutput,
        }
    }

    pub fn into_any_element(self) -> AnyElement {
        match self {
            Self::Logs(entity) => entity.into_any_element(),
            Self::Mods(entity) => entity.into_any_element(),
            Self::ResourcePacks(entity) => entity.into_any_element(),
            Self::Shaders(entity) => entity.into_any_element(),
            Self::Settings(entity) => entity.into_any_element(),
            Self::LiveGameOutput(entity) => entity.into_any_element(),
        }
    }
}
