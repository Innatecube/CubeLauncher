use std::sync::{Arc, atomic::{AtomicIsize, Ordering}};

use bridge::{account::Account, handle::BackendHandle, message::MessageToBackend};
use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, StyledExt, WindowExt, button::{Button, ButtonVariants}, h_flex, input::{Input, InputState}, sheet::Sheet, v_flex,
};
use rand::Rng;
use uuid::Uuid;

use crate::{component::shrinking_text::ShrinkingText, entity::{DataEntities, account::{AccountEntries, AccountExt}}, icon::PandoraIcon, interface_config::InterfaceConfig, png_render_cache};

use schema::unique_bytes::UniqueBytes;

const ACCOUNT_HEIGHT: f32 = 56.0;
const ACCOUNT_GAP: f32 = 8.0;
const ACCOUNT_STRIDE: f32 = ACCOUNT_HEIGHT + ACCOUNT_GAP;

#[allow(dead_code)]
struct AccountDragInfo {
    index: usize,
    name: SharedString,
    head: Option<UniqueBytes>,
    hide_skins: bool,
    badge_text: &'static str,
    start_y: f32,
    delta: AtomicIsize,
    backend_handle: BackendHandle,
}

impl Drop for AccountDragInfo {
    fn drop(&mut self) {
        let delta = self.delta.load(Ordering::Relaxed);
        if delta != 0 {
            self.backend_handle.send(MessageToBackend::ReorderAccounts {
                from_index: self.index,
                delta
            });
        }
    }
}

struct AnimateMove {
    from: usize,
    to: usize,
}

#[allow(dead_code)]
struct AccountDragPreview {
    name: SharedString,
    head: Option<UniqueBytes>,
    hide_skins: bool,
    badge_text: &'static str,
    start_x: f32,
}

impl Render for AccountDragPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let head = if self.hide_skins {
            gpui::img(ImageSource::Resource(Resource::Embedded("images/hidden_head.png".into())))
        } else if let Some(head) = &self.head {
            let resize = png_render_cache::ImageTransformation::Resize { width: 32, height: 32 };
            png_render_cache::render_with_transform(head.clone(), resize, cx)
        } else {
            gpui::img(ImageSource::Resource(Resource::Embedded("images/default_head.png".into())))
        };

        let badge_color = match self.badge_text {
            "Ely.by" => cx.theme().info,
            "Offline" => cx.theme().muted_foreground,
            _ => cx.theme().accent_foreground,
        };

        div()
            .absolute()
            .child(
                h_flex()
                    .left(px(-24.0))
                    .top(px(-22.0))
                    .w(px(340.0))
                    .h(px(ACCOUNT_HEIGHT))
                    .px_3()
                    .gap_3()
                    .items_center()
                    .rounded_none()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().popover)
                    .shadow_xl()
                    .opacity(0.96)
                    .child(head.size_8().min_w_8().min_h_8())
                    .child(
                        h_flex()
                            .min_w_0()
                            .flex_1()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_semibold()
                                    .text_color(cx.theme().popover_foreground)
                                    .child(ShrinkingText::new(self.name.clone()))
                            )
                            .child(
                                div()
                                    .text_size(rems(0.6875))
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_none()
                                    .bg(cx.theme().accent.opacity(0.3))
                                    .text_color(badge_color)
                                    .child(self.badge_text)
                            )
                    )
            )
    }
}

struct Accounts {
    backend_handle: BackendHandle,
    accounts: Entity<AccountEntries>,
    animate_move: Option<AnimateMove>,
    shift_amounts: Vec<f32>,
    last_accounts: Arc<[Account]>
}

pub fn build_accounts_sheet(data: &DataEntities, _: &mut Window, cx: &mut App) -> impl Fn(Sheet, &mut Window, &mut App) -> Sheet + 'static {
    let accounts = cx.new(|_| {
        let accounts = Accounts {
            backend_handle: data.backend_handle.clone(),
            accounts: data.accounts.clone(),
            animate_move: None,
            shift_amounts: Vec::new(),
            last_accounts: Arc::new([]),
        };

        accounts
    });

    move |sheet, _, _| {
        sheet
            .title(t::account::title())
            .size(px(480.))
            .when(cfg!(target_os = "macos"), |this| this.pt_5())
            .child(accounts.clone())
    }
}

impl Render for Accounts {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let hide_skins = InterfaceConfig::get(cx).hide_skins;

        let (accounts, selected_account) = {
            let accounts = self.accounts.read(cx);
            (accounts.accounts.clone(), accounts.selected_account_uuid)
        };

        let start_y = window.mouse_position().y.as_f32();

        if let Some(animate_move) = &self.animate_move {
            let reset = if animate_move.from != animate_move.to {
                !Arc::ptr_eq(&accounts, &self.last_accounts)
            } else {
                !cx.has_active_drag()
            };
            if reset {
                self.shift_amounts.clear();
                self.animate_move = None;
            }
        }
        self.last_accounts = accounts.clone();

        let items = accounts.iter().enumerate().filter_map(|(account_index, account)| {
            let head = if hide_skins {
                gpui::img(ImageSource::Resource(Resource::Embedded("images/hidden_head.png".into())))
            } else if let Some(head) = &account.head {
                let resize = png_render_cache::ImageTransformation::Resize { width: 32, height: 32 };
                png_render_cache::render_with_transform(head.clone(), resize, cx)
            } else {
                gpui::img(ImageSource::Resource(Resource::Embedded("images/default_head.png".into())))
            };
            let account_name = account.username(InterfaceConfig::get(cx).hide_usernames);

            let selected = Some(account.uuid) == selected_account;
            let is_elyby = account.is_elyby;
            let is_offline = account.offline;
            let group_name: SharedString = format!("account-row-{account_index}").into();
            let uuid = account.uuid;
            let (badge_text, badge_color) = if is_elyby {
                ("Ely.by", cx.theme().info)
            } else if is_offline {
                ("Offline", cx.theme().muted_foreground)
            } else {
                ("Microsoft", cx.theme().accent_foreground)
            };

            let row_bg = if selected {
                cx.theme().info.opacity(0.2)
            } else {
                cx.theme().input_background()
            };
            let row_border = if selected {
                cx.theme().info
            } else {
                cx.theme().border.opacity(0.55)
            };
            let hover_bg = if selected {
                cx.theme().info_hover.opacity(0.3)
            } else {
                cx.theme().secondary_hover
            };

            let mut shift = false;
            let mut shift_offset = 0.0;
            if let Some(animate_move) = &self.animate_move {
                if animate_move.from == account_index {
                    return None;
                }
                shift = if account_index > animate_move.from {
                    shift_offset = ACCOUNT_STRIDE;
                    account_index > animate_move.to
                } else {
                    account_index >= animate_move.to
                };
            }
            let previous_shift_amount = self.shift_amounts.get(account_index).copied().unwrap_or(0.0);
            let desired_shift_amount = if shift {
                ACCOUNT_STRIDE - shift_offset
            } else {
                0.0 - shift_offset
            };
            let shift_amount_delta = previous_shift_amount - desired_shift_amount;
            let new_shift_amount = if shift_amount_delta.abs() < 0.5 {
                desired_shift_amount
            } else {
                previous_shift_amount*0.75 + desired_shift_amount*0.25
            };

            if previous_shift_amount != new_shift_amount {
                window.request_animation_frame();
                if self.shift_amounts.len() < account_index+1 {
                    self.shift_amounts.resize(account_index+1, 0.0);
                }
                self.shift_amounts[account_index] = new_shift_amount;
            }

            Some(h_flex()
                .id(("account-row", account_index))
                .group(group_name.clone())
                .w_full()
                .h(px(ACCOUNT_HEIGHT))
                .top(px(new_shift_amount + shift_offset))
                .px_3()
                .gap_3()
                .rounded_none()
                .border_1()
                .border_color(if selected { cx.theme().info.opacity(0.7) } else { row_border })
                .bg(row_bg)
                .text_size(rems(0.9375))
                .line_height(relative(1.25))
                .hover(|style| style.bg(hover_bg))
                .when(!selected, |this| {
                    this.on_click({
                        let backend_handle = self.backend_handle.clone();
                        move |_, _, _| {
                            backend_handle.send(MessageToBackend::SelectAccount { uuid });
                        }
                    })
                })
                .child(head.size_8().min_w_8().min_h_8())
                .child(
                    h_flex()
                        .min_w_0()
                        .flex_1()
                        .gap_2()
                        .items_center()
                        .child(div().font_semibold().text_color(if selected { cx.theme().info } else { cx.theme().foreground }).child(ShrinkingText::new(account_name.clone())))
                        .child(
                            div()
                                .text_size(rems(0.6875))
                                .font_semibold()
                                .px_1p5()
                                .py_0p5()
                                .rounded_none()
                                .bg(if badge_text == "Ely.by" {
                                    cx.theme().info.opacity(0.18)
                                } else if badge_text == "Microsoft" {
                                    gpui::rgb(0x2563eb).opacity(0.2).into()
                                } else {
                                    cx.theme().accent.opacity(0.3)
                                })
                                .text_color(badge_color)
                                .child(badge_text)
                        )
                )
                .child(
                    Button::new(("account-delete", account_index))
                        .invisible()
                        .group_hover(group_name.clone(), |style| style.visible())
                        .ghost()
                        .compact()
                        .icon(PandoraIcon::Trash2)
                        .h_8()
                        .w_8()
                        .danger()
                        .on_click({
                            let backend_handle = self.backend_handle.clone();
                            move |_, _, cx| {
                                cx.stop_propagation();
                                backend_handle.send(MessageToBackend::DeleteAccount { uuid });
                            }
                        })
                ).into_any_element())
        });

        v_flex()
            .gap(px(ACCOUNT_GAP))
            .h_full()
            .child(
                h_flex()
                    .gap_2()
                    .w_full()
                    .child(
                        Button::new("add-account")
                            .flex_1()
                            .h_9()
                            .info()
                            .icon(PandoraIcon::Plus)
                            .label("Microsoft")
                            .on_click({
                                let backend_handle = self.backend_handle.clone();
                                move |_, window, cx| {
                                    crate::root::start_new_account_login(&backend_handle, window, cx);
                                }
                            })
                    )
                    .child(
                        Button::new("add-elyby")
                            .flex_1()
                            .h_9()
                            .success()
                            .icon(PandoraIcon::Plus)
                            .label("Ely.by")
                            .on_click({
                                let backend_handle = self.backend_handle.clone();
                                move |_, window, cx| {
                                    crate::root::capture_backdrop_blur(window, cx);
                                    let name_input = cx.new(|cx| {
                                        InputState::new(window, cx)
                                    });
                                    let backend_handle = backend_handle.clone();
                                    window.open_dialog(cx, move |dialog, _, cx| {
                                        let username = name_input.read(cx).value();
                                        let valid_name = username.len() >= 1 && username.len() <= 16 &&
                                            username.as_bytes().iter().all(|c| *c > 32 && *c < 127);

                                        let backend_handle = backend_handle.clone();
                                        let mut add_button = Button::new("add-elyby-submit").label(t::account::add::submit()).disabled(!valid_name).on_click(move |_, window, cx| {
                                            window.close_all_dialogs(cx);

                                            let uuid: u128 = rand::thread_rng().r#gen();
                                            let uuid = (uuid & !0xF0000000000000000000) | 0x30000000000000000000;
                                            let uuid = Uuid::from_u128(uuid);

                                            backend_handle.send(MessageToBackend::AddElyByAccount {
                                                name: username.clone().into(),
                                                uuid,
                                            });
                                        });

                                        if valid_name {
                                            add_button = add_button.success();
                                        }

                                        dialog
                                            .w(px(420.0))
                                            .close_button(true)
                                            .overlay_closable(true)
                                            .title(t::account::add::elyby())
                                            .child(v_flex()
                                                .gap_3()
                                                .pt_1()
                                                .child(crate::labelled(t::account::name(), Input::new(&name_input)))
                                                .child(add_button)
                                            )
                                    });
                                }
                            })
                    )
                    .child(
                        Button::new("add-offline")
                            .flex_1()
                            .h_9()
                            .ghost()
                            .border_1()
                            .icon(PandoraIcon::Plus)
                            .label("Offline")
                            .on_click({
                                let backend_handle = self.backend_handle.clone();
                                move |_, window, cx| {
                                    crate::root::capture_backdrop_blur(window, cx);
                                    let name_input = cx.new(|cx| {
                                        InputState::new(window, cx)
                                    });
                                    let uuid_input = cx.new(|cx| {
                                        InputState::new(window, cx).placeholder(t::account::uuid_random())
                                    });
                                    let backend_handle = backend_handle.clone();
                                    window.open_dialog(cx, move |dialog, _, cx| {
                                        let username = name_input.read(cx).value();
                                        let valid_name = username.len() >= 1 && username.len() <= 16 &&
                                            username.as_bytes().iter().all(|c| *c > 32 && *c < 127);
                                        let uuid = uuid_input.read(cx).value();
                                        let valid_uuid = uuid.is_empty() || Uuid::try_parse(&uuid).is_ok();

                                        let valid = valid_name && valid_uuid;

                                        let backend_handle = backend_handle.clone();
                                        let mut add_button = Button::new("add").label(t::account::add::submit()).disabled(!valid).on_click(move |_, window, cx| {
                                            window.close_all_dialogs(cx);

                                            let uuid = if let Ok(uuid) = Uuid::try_parse(&uuid) {
                                               uuid
                                            } else {
                                                let uuid: u128 = rand::thread_rng().r#gen();
                                                let uuid = (uuid & !0xF0000000000000000000) | 0x30000000000000000000; // set version to 3
                                                Uuid::from_u128(uuid)
                                            };

                                            backend_handle.send(MessageToBackend::AddOfflineAccount {
                                                name: username.clone().into(),
                                                uuid
                                            });
                                        });

                                        if valid {
                                            add_button = add_button.success();
                                        }

                                        dialog
                                            .w(px(420.0))
                                            .close_button(true)
                                            .overlay_closable(true)
                                            .title(t::account::add::offline())
                                            .child(v_flex()
                                                .gap_3()
                                                .pt_1()
                                                .child(crate::labelled(t::account::name(), Input::new(&name_input)))
                                                .child(crate::labelled(t::account::uuid(), Input::new(&uuid_input)))
                                                .child(add_button)
                                            )
                                    });
                                }
                            })
                    )
            )
            .children(items)
            .on_drag_move(cx.listener(move |this, event: &DragMoveEvent<AccountDragInfo>, window, cx| {
                if cx.active_drag_cursor_style() != Some(CursorStyle::ClosedHand) {
                    cx.set_active_drag_cursor_style(CursorStyle::ClosedHand, window);
                }

                let max_delta = this.accounts.read(cx).accounts.len().saturating_sub(1) as isize;

                let drag = event.drag(cx);
                let from_index = drag.index as isize;

                let delta = event.event.position.y.as_f32() - drag.start_y;
                let relative_delta = delta / ACCOUNT_STRIDE;
                let item_delta = relative_delta.round() as isize;
                let item_delta = item_delta.clamp(-from_index, max_delta - from_index);

                drag.delta.store(item_delta, Ordering::Relaxed);

                let to = (from_index + item_delta) as usize;

                if let Some(animate_move) = &this.animate_move {
                    if animate_move.to == to {
                        return;
                    }
                }

                this.animate_move = Some(AnimateMove { from: from_index as usize, to });
                cx.notify();
            }))
    }
}
