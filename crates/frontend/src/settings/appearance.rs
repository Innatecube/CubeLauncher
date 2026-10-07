use std::rc::Rc;

use gpui::*;
use gpui_component::{ActiveTheme, ThemeRegistry, button::{Button, ButtonVariants}, select::{Select, SelectEvent}};

use crate::{component::named_dropdown::{DropdownName, NamedDropdownItem, SearchableNamedDropdown}, entity::DataEntities, icon::PandoraIcon, interface_config::InterfaceConfig, settings::{SettingGroup, SettingItem, SettingItemWidget, SettingPage}};

pub(super) fn create_page(data: &DataEntities, window: &mut Window, cx: &mut App) -> SettingPage {
    SettingPage {
        title: t::settings::appearance,
        groups: vec![
            SettingGroup {
                title: Some(t::settings::appearance::theme),
                items: vec![
                    SettingItem {
                        title: t::settings::appearance::theme::base_theme,
                        description: t::settings::appearance::theme::base_theme_desc,
                        widget: create_theme_dropdown(window, cx),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::appearance::theme::open_theme_folder,
                        description: t::settings::appearance::theme::open_theme_folder_desc,
                        widget: SettingItemWidget::Any(Rc::new({
                            let theme_folder = data.theme_folder.clone();
                            move |_, _| {
                                Button::new("open-theme-folder").info().icon(PandoraIcon::FolderOpen).label(t::settings::appearance::theme::open_theme_folder()).on_click({
                                    let theme_folder = theme_folder.clone();
                                    move |_, window, cx| {
                                        crate::open_folder(&theme_folder, window, cx);
                                    }
                                }).into_any_element()
                            }
                        })),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::appearance::theme::font,
                        description: t::settings::appearance::theme::font_desc,
                        widget: create_font_dropdown(window, cx),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::appearance::theme::font_size,
                        description: t::settings::appearance::theme::font_size_desc,
                        widget: SettingItemWidget::Integer {
                            get: |cx| cx.theme().font_size.as_f32().round() as i32,
                            set: |val, cx| {
                                InterfaceConfig::get_mut(cx).set_font_size(val);
                                InterfaceConfig::apply_theme(cx, true);
                            },
                            min: Some(8),
                            max: Some(32),
                        },
                        ..Default::default()
                    },
                ].into(),
                searched_items: None
            },
            SettingGroup {
                title: Some(t::settings::appearance::window),
                items: vec![
                    SettingItem {
                        title: t::settings::appearance::window::use_os_titlebar,
                        description: t::settings::appearance::window::use_os_titlebar_desc,
                        widget: SettingItemWidget::Switch(|cfg| cfg.use_os_titlebar, |cfg, val| cfg.use_os_titlebar = val),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::appearance::window::hide_main_window,
                        description: t::settings::appearance::window::hide_main_window_desc,
                        widget: SettingItemWidget::Switch(|cfg| cfg.hide_main_window_on_launch, |cfg, val| cfg.hide_main_window_on_launch = val),
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::appearance::window::quit_when_main_window_closed,
                        description: t::settings::appearance::window::quit_when_main_window_closed_desc,
                        widget: SettingItemWidget::Switch(|cfg| cfg.quit_on_main_closed, |cfg, val| cfg.quit_on_main_closed = val),
                        ..Default::default()
                    },
                ].into(),
                searched_items: None
            },
            SettingGroup {
                title: Some(t::settings::appearance::sidebar),
                items: vec![
                    SettingItem {
                        title: || "Sidebar Style",
                        description: || "Choose between compact icons-only or expanded sidebar with names",
                        widget: create_sidebar_mode_dropdown(window, cx),
                        ..Default::default()
                    },
                    SettingItem {
                        title: || "Sidebar Width",
                        description: || "Exact width of the sidebar in pixels (64 for compact, 220 for expanded)",
                        widget: SettingItemWidget::Integer {
                            get: |cx| {
                                let w = InterfaceConfig::get(cx).sidebar_width;
                                if w <= 0.0 { crate::ui::DEFAULT_SIDEBAR_WIDTH as i32 } else { w.round() as i32 }
                            },
                            set: |val, cx| {
                                InterfaceConfig::get_mut(cx).sidebar_width = val as f32;
                                cx.refresh_windows();
                            },
                            min: Some(64),
                            max: Some(360),
                        },
                        ..Default::default()
                    },
                    SettingItem {
                        title: t::settings::appearance::sidebar::show_sidebar_icons,
                        description: t::settings::appearance::sidebar::show_sidebar_icons_desc,
                        widget: SettingItemWidget::Switch(|cfg| cfg.show_sidebar_icons, |cfg, val| cfg.show_sidebar_icons = val),
                        ..Default::default()
                    },
                ].into(),
                searched_items: None
            },
        ].into(),
        searched_groups: None,
        custom_render: None,
    }
}

fn create_theme_dropdown(window: &mut Window, cx: &mut App) -> SettingItemWidget {
    let themes = ThemeRegistry::global(cx).sorted_themes()
        .iter()
        .map(|theme| NamedDropdownItem {
            name: DropdownName::Literal(theme.name.clone()),
            item: theme.name.clone()
        })
        .collect();

    let initial = cx.theme().theme_name().clone();
    let dropdown = SearchableNamedDropdown::create_and_select(themes, initial, window, cx);

    cx.subscribe(&dropdown, |_, event: &SelectEvent<_>, cx| {
        let SelectEvent::Confirm(Some(theme_name)) = event else {
            return
        };
        InterfaceConfig::get_mut(cx).set_active_theme(theme_name.clone());
        InterfaceConfig::apply_theme(cx, true);
    }).detach();

    SettingItemWidget::Any(Rc::new(move |_, _| {
        Select::new(&dropdown).menu_width(px(200.0)).into_any_element()
    }))
}

fn create_font_dropdown(window: &mut Window, cx: &mut App) -> SettingItemWidget {
    let fonts = cx.text_system().all_font_names()
        .iter().filter(|name| !name.starts_with('.')).map(|font_name| {
            let font_name = SharedString::from(font_name);
            NamedDropdownItem { name: font_name.clone().into(), item: font_name }
        }).collect();

    let initial = cx.theme().font_family.clone();
    let dropdown = SearchableNamedDropdown::create_and_select(fonts, initial, window, cx);

    cx.subscribe(&dropdown, |_, event: &SelectEvent<_>, cx| {
        let SelectEvent::Confirm(Some(font_family)) = event else {
            return
        };

        InterfaceConfig::get_mut(cx).set_font_family(font_family.clone());
        InterfaceConfig::apply_theme(cx, true);
    }).detach();

    SettingItemWidget::Any(Rc::new(move |_, _| {
        Select::new(&dropdown).menu_width(px(200.0)).into_any_element()
    }))
}

fn create_sidebar_mode_dropdown(window: &mut Window, cx: &mut App) -> SettingItemWidget {
    let modes = vec![
        NamedDropdownItem {
            name: DropdownName::Literal("Expanded (Icons & Names)".into()),
            item: SharedString::from("expanded"),
        },
        NamedDropdownItem {
            name: DropdownName::Literal("Compact (Icons Only)".into()),
            item: SharedString::from("compact"),
        },
        NamedDropdownItem {
            name: DropdownName::Literal("Wide (260px)".into()),
            item: SharedString::from("wide"),
        },
    ];

    let current_width = InterfaceConfig::get(cx).sidebar_width;
    let initial: SharedString = if current_width == crate::ui::ICONIZED_SIDEBAR_WIDTH {
        "compact".into()
    } else if current_width >= 250.0 {
        "wide".into()
    } else {
        "expanded".into()
    };

    let dropdown = SearchableNamedDropdown::create_and_select(modes, initial, window, cx);

    cx.subscribe(&dropdown, |_, event: &SelectEvent<_>, cx| {
        let SelectEvent::Confirm(Some(mode)) = event else {
            return;
        };
        let new_width = match mode.as_str() {
            "compact" => crate::ui::ICONIZED_SIDEBAR_WIDTH,
            "wide" => 260.0,
            _ => crate::ui::DEFAULT_SIDEBAR_WIDTH,
        };
        InterfaceConfig::get_mut(cx).sidebar_width = new_width;
        cx.refresh_windows();
    }).detach();

    SettingItemWidget::Any(Rc::new(move |_, _| {
        Select::new(&dropdown).menu_width(px(200.0)).into_any_element()
    }))
}
