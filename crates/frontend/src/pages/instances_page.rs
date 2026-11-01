use bridge::handle::BackendHandle;
use gpui::{prelude::*, *};
use gpui_component::{
    IndexPath, button::{Button, ButtonCustomVariant, ButtonVariants}, h_flex, input::{Input, InputEvent, InputState}, scroll::ScrollableElement, select::{Select, SelectDelegate, SelectEvent, SelectItem, SelectState}, table::{DataTable, TableState}, v_flex
};
use strum::IntoEnumIterator;

use crate::{
    component::{instance_list::InstanceList, named_dropdown::{NamedDropdown, NamedDropdownItem}}, entity::{DataEntities, instance::InstanceEntries, metadata::FrontendMetadata}, icon::PandoraIcon, interface_config::{InstancesViewMode, InterfaceConfig}, pages::page::Page,
};

pub struct InstancesPage {
    instance_table: Entity<TableState<InstanceList>>,
    view_dropdown: Entity<SelectState<NamedDropdown<InstancesViewMode>>>,
    search_state: Entity<InputState>,

    metadata: Entity<FrontendMetadata>,
    instances: Entity<InstanceEntries>,
    tasks: Entity<crate::entity::task_manager::TaskManager>,

    backend_handle: BackendHandle,
}

impl InstancesPage {
    pub fn new(data: &DataEntities, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let instance_table = InstanceList::create_table(data, window, cx);
        let view_dropdown = cx.new(|cx| {
            let items = InstancesViewMode::iter().map(|view| {
                NamedDropdownItem { name: view.name(), item: view }
            }).collect::<Vec<_>>();
            let current_view = InterfaceConfig::get(cx).instances_view_mode;
            let row = items.iter().position(|v| v.item == current_view).unwrap_or(0);
            let delegate = NamedDropdown::new(items);
            SelectState::new(delegate, Some(IndexPath::new(row)), window, cx)
        });
        cx.subscribe(&view_dropdown, |_, _, event: &SelectEvent<NamedDropdown<InstancesViewMode>>, cx| {
            let SelectEvent::Confirm(Some(view)) = event else {
                return;
            };
            InterfaceConfig::get_mut(cx).instances_view_mode = *view;
        }).detach();

        let search_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search instances...")
                .clean_on_escape()
        });

        cx.subscribe_in(&search_state, window, |this, _, event: &InputEvent, _window, cx| {
            if let InputEvent::Change = event {
                let query = this.search_state.read(cx).text().to_string();
                this.instance_table.update(cx, |table, cx| {
                    table.delegate_mut().set_search_query(query);
                    cx.notify();
                });
            }
        }).detach();

        Self {
            instance_table,
            view_dropdown,
            search_state,
            metadata: data.metadata.clone(),
            instances: data.instances.clone(),
            tasks: data.tasks.clone(),
            backend_handle: data.backend_handle.clone(),
        }
    }
}

impl Page for InstancesPage {
    fn controls(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        gpui::Empty
    }

    fn scrollable(&self, _cx: &App) -> bool {
        false
    }
}

impl Render for InstancesPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view_mode = InterfaceConfig::get(cx).instances_view_mode;
        let view_element = match view_mode {
            InstancesViewMode::Cards => {
                let cards = self.instance_table.update(cx, |table, cx| {
                    table.delegate().render_cards(cx)
                });
                div()
                    .id("instances_cards_scroll")
                    .size_full()
                    .overflow_y_scrollbar()
                    .child(
                        div()
                            .w_full()
                            .p_4()
                            .child(cards)
                    )
                    .into_any_element()
            },
            InstancesViewMode::List => {
                DataTable::new(&self.instance_table).bordered(false).into_any_element()
            },
        };

        let custom_create = ButtonCustomVariant::new(cx)
            .color(rgb(0x04242e).into())
            .foreground(rgb(0x00f0ff).into())
            .hover(rgb(0x063e4d).into())
            .active(rgb(0x031c24).into());

        let create_instance = Button::new("create_instance")
            .custom(custom_create)
            .border_1()
            .border_color(rgb(0x00e5ff))
            .icon(PandoraIcon::Plus)
            .label(t::instance::create())
            .on_click(cx.listener(|this, _, window, cx| {
                crate::modals::create_instance::open_create_instance(this.metadata.clone(), this.instances.clone(),
                    this.tasks.clone(), this.backend_handle.clone(), window, cx);
            }));

        let select_view = div()
            .child(Select::new(&self.view_dropdown).title_prefix(format!("{}: ", t::instance::view_mode())));

        let top_bar = h_flex()
            .w_full()
            .flex_none()
            .gap_3()
            .items_center()
            .px_4()
            .pt_3()
            .pb_1()
            .child(
                div().w(px(220.0)).child(
                    Input::new(&self.search_state)
                        .prefix(PandoraIcon::Search)
                )
            )
            .child(create_instance)
            .child(div().ml_auto().child(select_view));

        v_flex()
            .size_full()
            .child(top_bar)
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .relative()
                    .with_animation(
                        ("instances_view_mode", view_mode as usize),
                        Animation::new(std::time::Duration::from_millis(180))
                            .with_easing(ease_out_quint()),
                        |div, delta| div.opacity(delta),
                    )
                    .child(view_element)
            )
    }
}

#[derive(Default)]
pub struct VersionList {
    pub versions: Vec<SharedString>,
    pub matched_versions: Vec<SharedString>,
}

impl SelectDelegate for VersionList {
    type Item = SharedString;

    fn items_count(&self, _section: usize) -> usize {
        self.matched_versions.len()
    }

    fn item(&self, ix: IndexPath) -> Option<&Self::Item> {
        self.matched_versions.get(ix.row)
    }

    fn position<V>(&self, value: &V) -> Option<IndexPath>
    where
        Self::Item: gpui_component::select::SelectItem<Value = V>,
        V: PartialEq,
    {
        for (ix, item) in self.matched_versions.iter().enumerate() {
            if item.value() == value {
                return Some(IndexPath::default().row(ix));
            }
        }

        None
    }

    fn perform_search(&mut self, query: &str, _window: &mut Window, _: &mut App) -> Task<()> {
        let lower_query = query.to_lowercase();

        self.matched_versions = self
            .versions
            .iter()
            .filter(|item| item.to_lowercase().starts_with(&lower_query))
            .cloned()
            .collect();

        Task::ready(())
    }
}
