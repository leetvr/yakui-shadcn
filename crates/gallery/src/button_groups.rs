//! Useful first-party toolbar and joined-input compositions from the pinned source.
use ui::{Button as B, ButtonSize as S, Icon as I, MenuItem as M, Variant as V};
use yakui_adequate_ui as ui;
pub const VARIANTS: &[&str] = &[
    "default",
    "orientation",
    "size",
    "separator",
    "split",
    "input",
    "dropdown",
    "popover",
];
fn action(label: &str, size: S, state: &mut crate::examples::ExampleState) {
    if B::new(label).variant(V::Outline).size(size).show().clicked {
        state.count += 1;
    }
}
pub fn render(variant: &str, state: &mut crate::examples::ExampleState, disabled: bool) {
    match variant {
        "default" => ui::button_group(|| {
            ui::button_group(|| {
                if B::new("")
                    .variant(V::Outline)
                    .size(S::Icon)
                    .icon(I::ArrowLeft)
                    .disabled(disabled)
                    .show()
                    .clicked
                {
                    state.count += 1;
                }
            });
            ui::button_group(|| {
                action("Archive", S::Default, state);
                action("Report", S::Default, state);
            });
            ui::button_group(|| {
                action("Snooze", S::Default, state);
                state.trigger = ui::dropdown_menu_trigger(&mut state.menu, || {
                    B::new("")
                        .variant(V::Outline)
                        .size(S::Icon)
                        .icon(I::MoreHorizontal)
                        .disabled(disabled)
                        .show()
                });
            });
        }),
        "orientation" => ui::ButtonGroup::new()
            .orientation(ui::Orientation::Vertical)
            .show(|| {
                for icon in [I::Plus, I::Minus] {
                    if B::new("")
                        .variant(V::Outline)
                        .size(S::Icon)
                        .icon(icon)
                        .disabled(disabled)
                        .show()
                        .clicked
                    {
                        state.count += 1;
                    }
                }
            }),
        "size" => ui::column(32., || {
            for (label, size, icon_size) in [
                ("Small", S::Sm, S::IconSm),
                ("Default", S::Default, S::Icon),
                ("Large", S::Lg, S::IconLg),
            ] {
                ui::button_group(|| {
                    for text in [label, "Button", "Group"] {
                        action(text, size, state);
                    }
                    B::new("")
                        .variant(V::Outline)
                        .size(icon_size)
                        .icon(I::Plus)
                        .disabled(disabled)
                        .show();
                });
            }
        }),
        "separator" => ui::button_group(|| {
            B::new("Copy")
                .variant(V::Secondary)
                .size(S::Sm)
                .disabled(disabled)
                .show();
            ui::button_group_separator();
            B::new("Paste")
                .variant(V::Secondary)
                .size(S::Sm)
                .disabled(disabled)
                .show();
        }),
        "split" => ui::button_group(|| {
            B::new("Button")
                .variant(V::Secondary)
                .disabled(disabled)
                .show();
            ui::button_group_separator();
            B::new("")
                .variant(V::Secondary)
                .size(S::Icon)
                .icon(I::Plus)
                .disabled(disabled)
                .show();
        }),
        "input" => ui::button_group(|| {
            ui::width(240., || {
                ui::Input::new(&mut state.text)
                    .placeholder("Search...")
                    .disabled(disabled)
                    .show();
            });
            if B::new("")
                .variant(V::Outline)
                .icon(I::Search)
                .disabled(disabled)
                .show()
                .clicked
            {
                state.count += 1;
            }
        }),
        "dropdown" => ui::button_group(|| {
            action("Follow", S::Default, state);
            state.trigger = ui::dropdown_menu_trigger(&mut state.menu, || {
                B::new("")
                    .variant(V::Outline)
                    .icon(I::ChevronDown)
                    .disabled(disabled)
                    .show()
            });
        }),
        "popover" => ui::button_group(|| {
            B::new("Copilot")
                .variant(V::Outline)
                .icon(I::Bot)
                .disabled(disabled)
                .show();
            state.trigger = B::new("")
                .variant(V::Outline)
                .size(S::Icon)
                .icon(I::ChevronDown)
                .disabled(disabled)
                .show();
            if state.trigger.clicked {
                state.overlay.open_from(state.trigger);
            }
        }),
        _ => unreachable!("unconfigured button-group preview"),
    }
}
pub fn items(variant: &str) -> Vec<M> {
    if variant == "dropdown" {
        vec![
            M::action(1, "Mute Conversation").icon(I::VolumeOff),
            M::action(2, "Mark as Read").icon(I::Check),
            M::action(3, "Report Conversation").icon(I::AlertTriangle),
            M::action(4, "Block User").icon(I::UserRoundX),
            M::action(5, "Share Conversation").icon(I::Share),
            M::action(6, "Copy Conversation").icon(I::Copy),
            M::separator(),
            M::action(7, "Delete Conversation")
                .icon(I::Trash)
                .destructive(true),
        ]
    } else {
        vec![
            M::action(1, "Mark as Read").icon(I::MailCheck),
            M::action(2, "Archive").icon(I::Archive),
            M::separator(),
            M::action(3, "Snooze").icon(I::Clock),
            M::action(4, "Add to Calendar").icon(I::CalendarPlus),
            M::action(5, "Add to List").icon(I::ListFilter),
            M::submenu(
                6,
                "Label As...",
                vec![
                    M::radio(7, "Personal", 1, true),
                    M::radio(8, "Work", 1, false),
                    M::radio(9, "Other", 1, false),
                ],
            )
            .icon(I::Tag),
            M::separator(),
            M::action(10, "Trash").icon(I::Trash2).destructive(true),
        ]
    }
}
pub fn content(variant: &str, state: &mut crate::examples::ExampleState) {
    if variant == "popover" {
        ui::popover_with_options(
            &mut state.overlay,
            ui::PopoverOptions {
                width: 288.,
                align: ui::PopoverAlign::End,
                ..Default::default()
            },
            |_| {
                yakui::pad(yakui::widgets::Pad::all(2.), || {
                    ui::column(10., || {
                        ui::popover_header(|| {
                            ui::popover_title("Start a new task with Copilot");
                            ui::popover_description("Describe your task in natural language.");
                        });
                        ui::column(12., || {
                            ui::fill_width(|| {
                                ui::textarea(&mut state.second, "I need to...");
                            });
                            ui::field_description("Copilot will open a pull request for review.");
                        });
                    })
                });
            },
        );
    } else if matches!(variant, "default" | "dropdown") {
        if let Some(id) = ui::dropdown_menu_with_options(
            &mut state.menu,
            &mut state.group_items,
            ui::MenuOptions {
                width: if variant == "default" { 160. } else { 176. },
                align: ui::PopoverAlign::End,
                ..Default::default()
            },
        ) {
            state.count = id as u32;
        }
    }
}
