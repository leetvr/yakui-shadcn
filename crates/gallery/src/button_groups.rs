//! Useful first-party toolbar and joined-input compositions from the pinned source.
use ui::{Button as B, ButtonSize as S, Icon as I, MenuItem as M, Variant as V};
use yakui_shadcn as ui;
pub const VARIANTS: &[&str] = &[
    "default",
    "orientation",
    "size",
    "separator",
    "split",
    "input",
    "dropdown",
    "popover",
    "select",
    "nested",
    "input-group",
];
fn action(label: &str, size: S, state: &mut crate::examples::ExampleState) {
    if B::new(label).variant(V::Outline).size(size).show().clicked {
        state.count += 1;
    }
}
pub fn render(variant: &str, state: &mut crate::examples::ExampleState, disabled: bool) {
    match variant {
        "nested" | "input-group" => {
            let voice = variant == "input-group";
            let mut theme = (*ui::theme()).clone();
            if voice {
                theme.radius = 999.;
            }
            ui::with_theme(theme, || {
                ui::button_group(|| {
                    ui::button_group(|| {
                        if B::new("")
                            .variant(V::Outline)
                            .size(S::Icon)
                            .icon(I::Plus)
                            .disabled(disabled)
                            .show()
                            .clicked
                        {
                            state.count += 1;
                        }
                    });
                    ui::button_group(|| {
                        ui::width(212., || {
                            ui::InputGroup::new()
                                .disabled(disabled || (voice && state.voice_mode))
                                .show(|| {
                                    ui::input_group_input(
                                        &mut state.text,
                                        if voice && state.voice_mode {
                                            "Record and send audio..."
                                        } else {
                                            "Send a message..."
                                        },
                                    );
                                    ui::input_group_addon(ui::InputGroupAlign::InlineEnd, || {
                                        state.trigger = if voice {
                                            ui::tooltip_trigger(&mut state.tooltips[0], || {
                                                let mut theme = (*ui::theme()).clone();
                                                let active = state.voice_mode;
                                                if active {
                                                    theme.secondary = if theme.dark {
                                                        ui::Color::rgb(154, 52, 18)
                                                    } else {
                                                        ui::Color::rgb(255, 237, 213)
                                                    };
                                                    theme.secondary_foreground = if theme.dark {
                                                        ui::Color::rgb(255, 237, 213)
                                                    } else {
                                                        ui::Color::rgb(194, 65, 12)
                                                    };
                                                }
                                                let mut response = ui::ControlResponse::default();
                                                ui::with_theme(theme, || {
                                                    response = ui::input_group_button("")
                                                        .size(S::IconXs)
                                                        .icon(I::AudioLines)
                                                        .variant(if active {
                                                            V::Secondary
                                                        } else {
                                                            V::Ghost
                                                        })
                                                        .disabled(disabled)
                                                        .show()
                                                });
                                                response
                                            })
                                        } else {
                                            ui::tooltip_trigger_area(&mut state.tooltips[0], || {
                                                ui::icon_colored(
                                                    I::AudioLines,
                                                    16.,
                                                    ui::theme().muted_foreground,
                                                )
                                            })
                                        };
                                        if voice && state.trigger.clicked {
                                            state.voice_mode = !state.voice_mode;
                                        }
                                    });
                                });
                        })
                    });
                })
            });
        }
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
        "select" => ui::button_group(|| {
            ui::button_group(|| {
                let items = currency_items();
                let symbol = match state.select.selected {
                    Some(1) => "€",
                    Some(2) => "£",
                    _ => "$",
                };
                state.trigger = ui::Select::new(&items)
                    .value_text(symbol)
                    .disabled(disabled)
                    .show(&mut state.select);
                ui::width(240., || {
                    ui::Input::new(&mut state.text)
                        .placeholder("10.00")
                        .disabled(disabled)
                        .show();
                });
            });
            ui::button_group(|| {
                if B::new("")
                    .variant(V::Outline)
                    .size(S::Icon)
                    .icon(I::ArrowRight)
                    .disabled(disabled)
                    .show()
                    .clicked
                {
                    state.count += 1;
                }
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
    if matches!(variant, "nested" | "input-group") {
        ui::Tooltip::new().show(&mut state.tooltips[0], "Voice Mode");
        return;
    }
    if variant == "select" {
        ui::SelectContent::new(&currency_items())
            .align_item_with_trigger(false)
            .show(&mut state.select);
    } else if variant == "popover" {
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
fn currency_items() -> Vec<ui::SelectItem> {
    vec![
        ui::SelectItem::new(0, "$ US Dollar"),
        ui::SelectItem::new(1, "€ Euro"),
        ui::SelectItem::new(2, "£ British Pound"),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    use yakui::{
        Rect, Vec2, Yakui,
        event::Event,
        input::{KeyCode, MouseButton},
    };
    fn frame(y: &mut Yakui, state: &mut crate::examples::ExampleState) {
        y.start();
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            render("input-group", state, false)
        });
        content("input-group", state);
        y.finish();
    }
    fn key(y: &mut Yakui, key: KeyCode) {
        for down in [true, false] {
            y.handle_event(Event::KeyChanged {
                key,
                down,
                modifiers: None,
            });
        }
    }
    fn click_voice(y: &mut Yakui, state: &crate::examples::ExampleState) {
        let rect = y.layout_dom().get(state.trigger.id.unwrap()).unwrap().rect;
        y.handle_event(Event::CursorMoved(Some(rect.center())));
        for down in [true, false] {
            y.handle_event(Event::MouseButtonChanged {
                button: MouseButton::One,
                down,
            });
        }
    }
    #[test]
    fn voice_mode_disables_the_actual_editor_and_can_be_toggled_back() {
        let mut y = Yakui::new();
        let size = Vec2::new(600., 400.);
        y.set_surface_size(size);
        y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
        ui::add_fonts(&mut y);
        let mut state = crate::examples::ExampleState::default();
        frame(&mut y, &mut state);
        key(&mut y, KeyCode::Tab);
        frame(&mut y, &mut state);
        key(&mut y, KeyCode::Tab);
        frame(&mut y, &mut state);
        y.handle_event(Event::TextInput('x'));
        frame(&mut y, &mut state);
        assert_eq!(state.text, "x");
        click_voice(&mut y, &state);
        frame(&mut y, &mut state);
        frame(&mut y, &mut state);
        assert!(state.voice_mode);
        key(&mut y, KeyCode::Tab);
        frame(&mut y, &mut state);
        key(&mut y, KeyCode::Tab);
        frame(&mut y, &mut state);
        y.handle_event(Event::TextInput('y'));
        frame(&mut y, &mut state);
        assert_eq!(state.text, "x");
        click_voice(&mut y, &state);
        frame(&mut y, &mut state);
        frame(&mut y, &mut state);
        assert!(!state.voice_mode);
        fn editor(y: &Yakui, id: yakui::WidgetId) -> Option<yakui::WidgetId> {
            let node = y.dom().get(id)?;
            if node.widget.type_name().ends_with("editor::Editor") {
                return Some(id);
            }
            node.children.iter().find_map(|id| editor(y, *id))
        }
        let target = editor(&y, y.dom().root()).unwrap();
        let rect = y.layout_dom().get(target).unwrap().rect;
        y.handle_event(Event::CursorMoved(Some(rect.center())));
        for down in [true, false] {
            y.handle_event(Event::MouseButtonChanged {
                button: MouseButton::One,
                down,
            });
        }
        frame(&mut y, &mut state);
        y.handle_event(Event::TextInput('z'));
        frame(&mut y, &mut state);
        assert_eq!(state.text, "xz");
    }
}
