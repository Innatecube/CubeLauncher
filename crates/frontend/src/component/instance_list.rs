use std::sync::Arc;

use bridge::{instance::InstanceStatus, message::MessageToBackend};
use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Disableable, Sizable, StyledExt, button::{Button, ButtonCustomVariant, ButtonVariants, DropdownButton}, h_flex, menu::{DropdownMenu, PopupMenuItem}, separator::Separator, spinner::Spinner, table::{Column, ColumnSort, TableDelegate, TableState}, v_flex
};
use indexmap::IndexMap;

use crate::{
    component::{clipped_element::AnimatedClippedElement, reorderable::{Reorderable, ReorderableDragInfo, ReorderableState}, responsive_grid::ResponsiveGrid}, entity::{
        DataEntities, instance::{InstanceAddedEvent, InstanceEntry, InstanceModifiedEvent, InstanceRemovedEvent}
    }, icon::PandoraIcon, interface_config::InterfaceConfig, root, ui
};

pub struct InstanceList {
    columns: Vec<Column>,
    items: Vec<InstanceEntry>,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
    reorderable_state: Entity<ReorderableState>,
    data: DataEntities,
    _instance_added_subscription: Subscription,
    _instance_removed_subscription: Subscription,
    _instance_modified_subscription: Subscription,
}

impl InstanceList {
    pub fn create_table(data: &DataEntities, window: &mut Window, cx: &mut App) -> Entity<TableState<Self>> {
        let instances = data.instances.clone();
        let items: Vec<InstanceEntry> = instances.read(cx).entries.values().filter_map(|i| {
            let entry = i.read(cx).clone();
            if entry.name == schema::quickplay::INSTANCE_NAME {
                None
            } else {
                Some(entry)
            }
        }).collect();
        let filtered_indices = (0..items.len()).collect();
        cx.new(|cx| {
            let _instance_added_subscription = cx.subscribe::<_, InstanceAddedEvent>(&instances, |table: &mut TableState<InstanceList>, _, event, cx| {
                if event.instance.name == schema::quickplay::INSTANCE_NAME {
                    return;
                }
                table.delegate_mut().items.insert(0, event.instance.clone());
                table.delegate_mut().update_filtered();
                cx.notify();
            });
            let _instance_removed_subscription = cx.subscribe::<_, InstanceRemovedEvent>(&instances, |table, _, event, cx| {
                table.delegate_mut().items.retain(|instance| {
                    instance.id != event.id
                });
                table.delegate_mut().update_filtered();
                cx.notify();
            });
            let _instance_modified_subscription = cx.subscribe::<_, InstanceModifiedEvent>(&instances, |table, _, event, cx| {
                if let Some(entry) = table.delegate_mut().items.iter_mut().find(|entry| entry.id == event.instance.id) {
                    *entry = event.instance.clone();
                    table.delegate_mut().update_filtered();
                    cx.notify();
                }
            });
            let instance_list = Self {
                columns: vec![
                    Column::new("controls", "")
                        .width(150.)
                        .fixed_left()
                        .movable(false)
                        .resizable(false),
                    Column::new("name", t::instance::name())
                        .width(150.)
                        .fixed_left()
                        .sortable()
                        .resizable(true),
                    Column::new("version", t::instance::version())
                        .width(150.)
                        .fixed_left()
                        .sortable()
                        .resizable(true),
                    Column::new("loader", t::instance::modloader())
                        .width(150.)
                        .fixed_left()
                        .resizable(true),
                ],
                items,
                search_query: String::new(),
                filtered_indices,
                reorderable_state: ReorderableState::new(window, cx),
                data: data.clone(),
                _instance_added_subscription,
                _instance_removed_subscription,
                _instance_modified_subscription,
            };
            TableState::new(instance_list, window, cx)
        })
    }

    pub fn update_filtered(&mut self) {
        let q = self.search_query.to_lowercase();
        self.filtered_indices = self.items.iter().enumerate().filter_map(|(ix, item)| {
            if q.is_empty() || item.name.to_lowercase().contains(&q) {
                Some(ix)
            } else {
                None
            }
        }).collect();
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.update_filtered();
    }

    pub fn render_cards(&self, cx: &mut App) -> AnyElement {
        let mut by_group = IndexMap::<Option<Arc<str>>, Vec<AnyElement>>::default();

        for &ix in &self.filtered_indices {
            if let Some(item) = self.items.get(ix) {
                let group = item.configuration.group.clone();
                let rendered = self.render_card(item, cx);
                by_group.entry(group).or_default().push(rendered);
            }
        }

        let size = Size::new(
            gpui::AvailableSpace::MinContent,
            gpui::AvailableSpace::MinContent
        );
        if by_group.len() == 1 {
            ResponsiveGrid::new(size).w_full().gap(px(14.0)).children(by_group.into_iter().next().unwrap().1).into_any_element()
        } else {
            let size_without_no_group = if by_group.contains_key(&None) {
                by_group.len() - 1
            } else {
                by_group.len()
            };

            let ordering = &InterfaceConfig::get(cx).instance_group_order;

            by_group.sort_by_cached_key(|group_name, _| if let Some(group_name) = group_name {
                ordering.iter().position(|ordering_name| ordering_name == group_name).map(|v| v+1).unwrap_or(0)
            } else {
                std::usize::MAX
            });

            if size_without_no_group == 1 {
                v_flex()
                    .w_full()
                    .gap_2()
                    .children(by_group.into_iter().map(|(group_name, children)| {
                        let group_name = group_name.clone().map(SharedString::from).unwrap_or("No Group".into());
                        Self::render_card_group(group_name, children, cx, None)
                    }))
                    .into_any_element()
            } else {
                if !cx.has_active_drag() {
                    self.reorderable_state.update(cx, |state, cx| {
                        let Some((from, to)) = state.take_reorder() else {
                            return;
                        };

                        // Move element
                        let Some((key, value)) = by_group.shift_remove_index(from) else {
                            return
                        };
                        by_group.shift_insert(to, key, value);

                        // Update ordering
                        let order = &mut InterfaceConfig::get_mut(cx).instance_group_order;
                        order.clear();
                        for (key, _) in &by_group {
                            if let Some(key) = key {
                                order.push(key.clone());
                            }
                        }
                    });
                }

                let no_group_children = by_group.swap_remove(&None);

                let reorderable = Reorderable::new(&self.reorderable_state, by_group.len(), cx, move |render_index, info, _, cx| {
                    let Some((group_name, children)) = by_group.get_index_mut(render_index) else {
                        return div().into_any_element();
                    };
                    let Some(group_name) = group_name.clone().map(SharedString::from) else {
                        return div().into_any_element();
                    };
                    Self::render_card_group(group_name, std::mem::take(children), cx, Some(info))
                }).w_full().v_flex().gap_2().into_any_element();

                v_flex()
                    .w_full()
                    .gap_2()
                    .child(reorderable)
                    .when_some(no_group_children, |this, no_group_children| {
                        this.child(Self::render_card_group("No Group".into(), no_group_children, cx, None))
                    })
                    .into_any_element()
            }
        }
    }

    fn render_card_group(group_name: SharedString, children: Vec<AnyElement>, cx: &mut App, drag_info: Option<ReorderableDragInfo>) -> AnyElement {
        let size = Size::new(
            gpui::AvailableSpace::MinContent,
            gpui::AvailableSpace::MinContent
        );

        let open = !InterfaceConfig::get(cx).instance_groups_closed.contains(&group_name);

        v_flex()
            .child(h_flex()
                .pb_1()
                .gap_2()
                .when_some(drag_info, |this, drag_info| {
                    this.child(drag_info.create_grip(cx, {
                        let group_name = group_name.clone();
                        move |_: &ReorderableDragInfo, pos, _, cx| {
                            cx.new(|_| GroupDragPreview {
                                name: group_name.clone(),
                                start_x: pos.x.as_f32(),
                            })
                        }
                    }))
                })
                .child(group_name.clone())
                .child(h_flex()
                    .child(Button::new((group_name.clone(), 0x23B40E19))
                        .ghost()
                        .small()
                        .icon(PandoraIcon::Menu))
                    .child(Button::new((group_name.clone(), 0x23B40E18))
                        .ghost()
                        .small()
                        .icon(if open { PandoraIcon::ChevronDown } else { PandoraIcon::ChevronLeft })
                        .on_click({
                            let group_name = group_name.clone();
                            move |_, _, cx| {
                                if open {
                                    InterfaceConfig::get_mut(cx).instance_groups_closed.insert(group_name.clone());
                                } else {
                                    InterfaceConfig::get_mut(cx).instance_groups_closed.remove(&group_name);
                                }
                            }
                        })))
            )
            .child(Separator::horizontal().pb_2())
            .child(AnimatedClippedElement::new(
                (group_name, 0x23B40E17).into(),
                open,
                move |amount| {
                    ResponsiveGrid::new(size).w_full().gap(px(14.0)).children(children).opacity(amount).into_any_element()
                }
            ))
            .into_any_element()
    }

    fn render_card(&self, item: &InstanceEntry, cx: &mut App) -> AnyElement {
        let index = item.id.index;

        let active_task = self.data.tasks.read(cx).snapshots().into_iter().find(|t| !t.is_finished && (t.id == Some(item.id) || t.name == item.name));
        let download_pct = active_task.as_ref().map(|t| {
            if t.total > 0 {
                (t.count as f32 / t.total as f32).clamp(0.01, 1.0)
            } else {
                0.08
            }
        });

        let icon_tile = crate::component::instance_icon::render_instance_tile_with_download(
            item.icon.clone(),
            &item.name,
            item.configuration.instance_fallback_icon.as_deref(),
            80.0,
            18.0,
            download_pct,
            cx,
        );

        let play_button = if let Some(pct) = download_pct {
            let pct_int = (pct * 100.0) as u32;
            h_flex()
                .w_full()
                .h(px(28.0))
                .items_center()
                .justify_center()
                .gap_1p5()
                .text_xs()
                .font_medium()
                .text_color(rgb(0x94a3b8))
                .child(
                    div()
                        .size(px(14.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(0x00e5ff))
                        .child(PandoraIcon::Download)
                )
                .child(format!("Installing... {}%", pct_int))
                .into_any_element()
        } else {
            render_play_button(item, index, self.data.clone(), cx)
        };

        let menu = Button::new(("menu", index))
            .ghost()
            .small()
            .icon(PandoraIcon::Menu)
            .dropdown_menu_with_anchor(Anchor::TopRight, {
                let instance = item.clone();
                let data = self.data.clone();
                move |this, _window, _cx| {
                    this
                        .item(PopupMenuItem::new("Change Icon").on_click({
                            let instance_id = instance.id;
                            let backend_handle = data.backend_handle.clone();
                            move |_, window, cx| {
                                let backend_handle = backend_handle.clone();
                                crate::modals::select_icon::open_select_icon(Box::new(move |icon, _| {
                                    backend_handle.send(MessageToBackend::SetInstanceIcon {
                                        id: instance_id,
                                        icon: Some(icon),
                                    });
                                }), window, cx);
                            }
                        }))
                        .item(PopupMenuItem::new("Move to Group").on_click({
                            let instance_id = instance.id;
                            let instance_name = instance.name.clone();
                            let instances = data.instances.clone();
                            let backend_handle = data.backend_handle.clone();
                            move |_, window, cx| {
                                crate::modals::move_instance_to_group::open_move_instance_to_group_modal(instance_id,
                                    instance_name.clone(), instances.clone(), backend_handle.clone(), window, cx);
                            }
                        }))
                }
            });

        let theme = cx.theme();
        let card_instance_name = item.name.clone();
        v_flex()
            .id(("instance_card", index))
            .flex_1()
            .p_3()
            .gap_2()
            .w_full()
            .min_w(px(180.0))
            .max_w(px(260.0))
            .bg(theme.popover)
            .border_1()
            .border_color(theme.border)
            .rounded_none()
            .cursor_pointer()
            .hover(move |style| style.bg(rgb(0x131926)).border_color(rgb(0x00f0ff)).shadow_lg())
            .on_click({
                let name = card_instance_name.clone();
                move |_, window, cx| {
                    root::switch_page(ui::PageType::InstancePage { name: name.clone() },
                        &[ui::PageType::Instances], window, cx);
                }
            })
            .child(
                div()
                    .relative()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon_tile)
                    .child(
                        div().absolute().right_0().top_0().child(menu)
                    )
            )
            .child(
                v_flex()
                    .w_full()
                    .items_center()
                    .text_center()
                    .gap_1()
                    .child(
                        div()
                            .font_bold()
                            .text_base()
                            .text_color(rgb(0xffffff))
                            .truncate()
                            .w_full()
                            .child(item.name.clone())
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .justify_center()
                            .gap_1p5()
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_none()
                                    .bg(rgb(0x111724))
                                    .border_1()
                                    .border_color(rgb(0x1e2a40))
                                    .text_size(px(10.5))
                                    .font_semibold()
                                    .text_color(rgb(0x00e5ff))
                                    .child(item.configuration.loader.pretty_name())
                            )
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_none()
                                    .bg(rgb(0x111724))
                                    .border_1()
                                    .border_color(rgb(0x1e2a40))
                                    .text_size(px(10.5))
                                    .font_medium()
                                    .text_color(rgb(0x94a3b8))
                                    .child(item.configuration.minecraft_version.to_string())
                            )
                    )
            )
            .child(
                h_flex()
                    .w_full()
                    .child(play_button)
            )
            .into_any_element()
    }
}

impl TableDelegate for InstanceList {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.filtered_indices.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> gpui_component::table::Column {
        self.columns[col_ix].clone()
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: gpui_component::table::ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        if let Some(col) = self.columns.get_mut(col_ix) {
            match col.key.as_ref() {
                "name" => self.items.sort_by(|a, b| match sort {
                    ColumnSort::Descending => lexical_sort::natural_lexical_cmp(&a.name, &b.name).reverse(),
                    _ => lexical_sort::natural_lexical_cmp(&a.name, &b.name),
                }),
                "version" => self.items.sort_by(|a, b| match sort {
                    ColumnSort::Descending => lexical_sort::natural_lexical_cmp(&a.configuration.minecraft_version, &b.configuration.minecraft_version).reverse(),
                    _ => lexical_sort::natural_lexical_cmp(&a.configuration.minecraft_version, &b.configuration.minecraft_version),
                }),
                _ => {},
            }
        }
        self.update_filtered();
    }

    fn render_td(&mut self, row_ix: usize, col_ix: usize, _window: &mut Window, _cx: &mut Context<TableState<Self>>) -> impl IntoElement {
        let real_ix = self.filtered_indices.get(row_ix).copied().unwrap_or(row_ix);
        let item = &self.items[real_ix];
        if let Some(col) = self.columns.get(col_ix) {
            match col.key.as_ref() {
                "name" => item.name.clone().into_any_element(),
                "version" => item.configuration.minecraft_version.as_str().into_any_element(),
                "controls" => {
                    let play_button = render_play_button(item, real_ix, self.data.clone(), _cx);

                    h_flex()
                        .size_full()
                        .border_r_4()
                        .child(play_button)
                        .into_any_element()
                },
                "loader" => item.configuration.loader.pretty_name().into_any_element(),
                _ => t::common::unknown().into_any_element(),
            }
        } else {
            t::common::unknown().into_any_element()
        }
    }
}

fn render_play_button(item: &InstanceEntry, index: usize, data: DataEntities, cx: &App) -> AnyElement {
    let name = item.name.clone();
    let id = item.id;

    let custom_play = ButtonCustomVariant::new(cx)
        .color(rgb(0x04242e).into())
        .foreground(rgb(0x00f0ff).into())
        .hover(rgb(0x063e4d).into())
        .active(rgb(0x031c24).into());

    if data.tasks.read(cx).is_active(id) {
        return Button::new(("installing", index))
            .custom(custom_play)
            .icon(Spinner::new())
            .label("Installing...")
            .loading(true)
            .w_full()
            .small()
            .into_any_element();
    }

    match item.status {
        InstanceStatus::NotRunning => {
            Button::new(("start_instance", index))
                .custom(custom_play)
                .border_1()
                .border_color(rgb(0x00e5ff))
                .icon(PandoraIcon::Play)
                .label("Play")
                .w_full()
                .small()
                .on_click(
                move |_, window, cx| {
                    cx.stop_propagation();
                    root::start_instance(id, name.clone(), None, &data, window, cx);
                },
            )
            .into_any_element()
        },
        InstanceStatus::Launching => {
            let name = name.clone();
            let data = data.clone();
            DropdownButton::new(("launching_split", index))
                .warning()
                .rounded_none()
                .w_full()
                .small()
                .button(
                    Button::new(("launching", index))
                        .warning()
                        .icon(Spinner::new())
                        .label(t::instance::start::starting())
                        .loading(true)
                        .flex_1()
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
                                    cx.stop_propagation();
                                    crate::root::restart_instance(id, name.clone(), None, &data, window, cx);
                                }
                            });
                        let stop_item = PopupMenuItem::new(t::instance::kill_instance())
                            .icon(PandoraIcon::Close)
                            .on_click({
                                let backend_handle = data.backend_handle.clone();
                                move |_, _, cx| {
                                    cx.stop_propagation();
                                    backend_handle.send(MessageToBackend::KillInstance { id });
                                }
                            });
                        this.item(restart_item).item(stop_item)
                    }
                })
                .into_any_element()
        },
        InstanceStatus::Stopping => {
            Button::new(("stopping", index))
                .danger()
                .icon(Spinner::new())
                .label(t::instance::start::stopping())
                .loading(true)
                .w_full()
                .small()
                .on_click({
                    let backend_handle = data.backend_handle.clone();
                    move |_, _, cx| {
                        cx.stop_propagation();
                        backend_handle.send(MessageToBackend::KillInstance { id });
                    }
                })
                .into_any_element()
        },
        InstanceStatus::Running => {
            let name = name.clone();
            let data = data.clone();
            DropdownButton::new(("running_split", index))
                .danger()
                .rounded_none()
                .w_full()
                .small()
                .button(
                    Button::new(("kill_instance", index))
                        .danger()
                        .icon(PandoraIcon::Close)
                        .label(t::instance::kill())
                        .flex_1()
                        .on_click({
                            let backend_handle = data.backend_handle.clone();
                            move |_, _, cx| {
                                cx.stop_propagation();
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
                                move |_, _, cx| {
                                    cx.stop_propagation();
                                    backend_handle.send(MessageToBackend::KillInstance { id });
                                }
                            });
                        let restart_item = PopupMenuItem::new("Restart Instance")
                            .icon(PandoraIcon::RefreshCcw)
                            .on_click({
                                let data = data.clone();
                                let name = name.clone();
                                move |_, window, cx| {
                                    cx.stop_propagation();
                                    crate::root::restart_instance(id, name.clone(), None, &data, window, cx);
                                }
                            });
                        this.item(stop_item).item(restart_item)
                    }
                })
                .into_any_element()
        },
    }
}

pub struct GroupDragPreview {
    pub name: SharedString,
    pub start_x: f32,
}

impl Render for GroupDragPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .child(
                h_flex()
                    .left(px(-12.0))
                    .top(px(-12.0))
                    .px_3()
                    .py_2()
                    .gap_2()
                    .items_center()
                    .rounded_none()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().popover)
                    .text_color(cx.theme().popover_foreground)
                    .shadow_xl()
                    .opacity(0.96)
                    .child(
                        div()
                            .size_6()
                            .min_w_6()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(cx.theme().muted_foreground)
                            .child(PandoraIcon::GripVertical)
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .child(self.name.clone())
                    )
                    .child(
                        div()
                            .text_xs()
                            .px_1p5()
                            .py_0p5()
                            .rounded_none()
                            .bg(cx.theme().muted)
                            .text_color(cx.theme().muted_foreground)
                            .child("Group")
                    )
            )
    }
}

