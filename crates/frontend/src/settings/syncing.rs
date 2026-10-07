use std::rc::Rc;

use gpui::*;
use gpui_component::{scroll::ScrollableElement, v_flex};

use crate::{
    entity::DataEntities,
    pages::syncing_page::SyncingPage,
    settings::SettingPage,
};

pub(super) fn create_page(data: &DataEntities, window: &mut Window, cx: &mut App) -> SettingPage {
    let syncing_page = cx.new(|cx| SyncingPage::new(data, window, cx));

    SettingPage {
        title: t::instance::sync::label,
        groups: Rc::new([]),
        searched_groups: None,
        custom_render: Some(Rc::new(move |_window, _cx| {
            v_flex()
                .size_full()
                .min_h_0()
                .overflow_y_scrollbar()
                .child(syncing_page.clone())
                .into_any_element()
        })),
    }
}
