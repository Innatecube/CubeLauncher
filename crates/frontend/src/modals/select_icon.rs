use std::sync::Arc;

use bridge::message::EmbeddedOrRaw;
use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants}, scroll::ScrollableElement, v_flex, ActiveTheme as _, WindowExt
};
use parking_lot::RwLock;

use crate::icon::PandoraIcon;

pub static MODRINTH_ICONS: &[&'static str] = &[
    "backpack", "beacon", "blue-shark", "bookshelf", "brown-bear",
    "cake", "campfire", "chest", "cogwheel", "command-block",
    "cooking-pot", "couch", "crafting-table", "creeper", "enchanting-table",
    "ender-chest", "ender-dragon", "engine", "fabric", "forge",
    "furnace", "gizmo", "globe", "grass-block", "lantern",
    "moobloom", "mr-pack", "neoforge", "orb", "oxygen-distributor",
    "pancakes", "pickaxe", "poke-ball", "quilt", "redstone-block",
    "sculk-sensor", "skeleton", "skillet", "slime-block", "space-helmet",
    "sticky-piston", "sword", "terminal", "tiny-potato", "tire",
    "tnt", "wrench-rinth", "wrench", "zombie",
];

pub fn get_random_modrinth_icon() -> Option<EmbeddedOrRaw> {
    use rand::Rng;
    let icon_name = MODRINTH_ICONS[rand::thread_rng().gen_range(0..MODRINTH_ICONS.len())];
    let path = format!("instance_icons/{icon_name}.png");
    if let Ok(Some(bytes)) = crate::Assets.load(&path) {
        Some(EmbeddedOrRaw::Raw(bytes.to_vec().into()))
    } else {
        None
    }
}

pub fn open_select_icon(
    selected: Box<dyn FnOnce(EmbeddedOrRaw, &mut App)>,
    window: &mut Window,
    cx: &mut App,
) {
    let select_file_task = Arc::new(RwLock::new(Task::ready(())));
    let selected = Arc::new(RwLock::new(Some(selected)));
    window.open_dialog(cx, move |dialog, _, cx| {
        let theme = cx.theme();
        let border_color = theme.border;
        let secondary = theme.secondary;
        let primary = theme.primary;

        let icons = MODRINTH_ICONS.iter().enumerate().map(|(index, icon_name)| {
            let icon_name = *icon_name;
            let path = format!("instance_icons/{icon_name}.png");
            let selected = selected.clone();

            div()
                .id(index)
                .p_1()
                .rounded(px(6.0))
                .border_1()
                .border_color(border_color)
                .hover(move |s| s.bg(secondary).border_color(primary))
                .cursor_pointer()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    gpui::img(ImageSource::Resource(Resource::Embedded(path.clone().into())))
                        .size_12()
                )
                .on_click(move |_, window, cx| {
                    if let Some(selected) = selected.write().take() {
                        if let Ok(Some(bytes)) = crate::Assets.load(&path) {
                            (selected)(EmbeddedOrRaw::Raw(bytes.to_vec().into()), cx);
                        }
                    }
                    window.close_dialog(cx);
                })
        });

        let grid = div()
            .grid()
            .grid_cols(7)
            .w_full()
            .max_h_96()
            .gap_2()
            .overflow_y_scrollbar()
            .children(icons);

        let content = v_flex()
            .size_full()
            .gap_3()
            .child(
                Button::new("custom_icon")
                    .success()
                    .label(t::common::custom_icon())
                    .icon(PandoraIcon::File)
                    .on_click({
                        let selected = selected.clone();
                        let select_file_task = select_file_task.clone();
                        move |_, window, cx| {
                            let receiver = cx.prompt_for_paths(PathPromptOptions {
                                files: true,
                                directories: false,
                                multiple: false,
                                prompt: Some(t::instance::select_png_icon().into()),
                            });

                            let selected = selected.clone();
                            *select_file_task.write() = window.spawn(cx, async move |cx| {
                                let Ok(Ok(Some(result))) = receiver.await else {
                                    return;
                                };
                                let Some(path) = result.first() else {
                                    return;
                                };
                                let Ok(bytes) = std::fs::read(path) else {
                                    return;
                                };
                                _ = cx.update(move |window, cx| {
                                    if let Some(selected) = selected.write().take() {
                                        (selected)(EmbeddedOrRaw::Raw(bytes.into()), cx);
                                    }
                                    window.close_dialog(cx);
                                });
                            });
                        }
                    }),
            )
            .child(grid);

        dialog
            .title(t::instance::select_icon())
            .child(content)
    });
}
