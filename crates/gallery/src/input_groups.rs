//! Pinned source compositions built entirely from public library slots.
use ui::{Icon as I, InputGroupAlign as A};
use yakui_shadcn as ui;
pub const VARIANTS: &[&str] = &[
    "default",
    "icon",
    "text",
    "kbd",
    "spinner",
    "inline-start",
    "inline-end",
    "block-end",
];
fn icon(icon: I) {
    ui::icon_colored(icon, 16., ui::theme().muted_foreground);
}
fn addon(align: A, children: impl FnOnce()) {
    ui::input_group_addon(align, children);
}
pub fn render(variant: &str, state: &mut crate::examples::ExampleState, disabled: bool) {
    ui::width(
        if variant == "default" { 320. } else { 384. },
        || match variant {
            "default" | "kbd" => ui::InputGroup::new().disabled(disabled).show(|| {
                ui::input_group_input(&mut state.input_group_values[0], "Search...");
                addon(A::InlineStart, || icon(I::Search));
                addon(A::InlineEnd, || {
                    if variant == "kbd" {
                        ui::kbd("⌘K");
                    } else {
                        ui::input_group_text("12 results");
                    }
                });
            }),
            "icon" => ui::column(24., || {
                for (index, placeholder, leading, trailing) in [
                    (0, "Search...", Some(I::Search), None),
                    (1, "Enter your email", Some(I::Mail), None),
                    (2, "Card number", Some(I::CreditCard), Some(I::Check)),
                    (3, "Card number", None, Some(I::Star)),
                ] {
                    ui::InputGroup::new().disabled(disabled).show(|| {
                        ui::input_group_input(&mut state.input_group_values[index], placeholder);
                        if let Some(leading) = leading {
                            addon(A::InlineStart, || icon(leading));
                        }
                        if let Some(trailing) = trailing {
                            addon(A::InlineEnd, || {
                                icon(trailing);
                                if index == 3 {
                                    icon(I::Info);
                                }
                            });
                        }
                    });
                }
            }),
            "text" => ui::column(24., || {
                for (index, placeholder, prefix, suffix) in [
                    (0, "0.00", "$", "USD"),
                    (1, "example.com", "https://", ".com"),
                    (2, "Enter your username", "", "@company.com"),
                ] {
                    ui::InputGroup::new().disabled(disabled).show(|| {
                        if !prefix.is_empty() {
                            addon(A::InlineStart, || ui::input_group_text(prefix));
                        }
                        ui::input_group_input(&mut state.input_group_values[index], placeholder);
                        addon(A::InlineEnd, || ui::input_group_text(suffix));
                    });
                }
                ui::InputGroup::new().disabled(disabled).show(|| {
                    ui::input_group_textarea(
                        &mut state.input_group_values[3],
                        "Enter your message",
                    );
                    addon(A::BlockEnd, || ui::input_group_text("120 characters left"));
                });
            }),
            "spinner" => ui::column(16., || {
                for (index, placeholder) in [
                    "Searching...",
                    "Processing...",
                    "Saving changes...",
                    "Refreshing data...",
                ]
                .into_iter()
                .enumerate()
                {
                    ui::InputGroup::new().disabled(disabled).show(|| {
                        ui::input_group_input(&mut state.input_group_values[index], placeholder);
                        addon(
                            if index == 1 || index == 3 {
                                A::InlineStart
                            } else {
                                A::InlineEnd
                            },
                            || {
                                if index == 2 {
                                    ui::input_group_text("Saving...");
                                }
                                if index == 3 {
                                    icon(I::Loader);
                                } else {
                                    ui::spinner(16., state.phase);
                                }
                            },
                        );
                        if index == 3 {
                            addon(A::InlineEnd, || ui::input_group_text("Please wait..."));
                        }
                    });
                }
            }),
            "inline-start" | "inline-end" => ui::column(12., || {
                ui::label("Input");
                ui::InputGroup::new().disabled(disabled).show(|| {
                    ui::Input::new(&mut state.input_group_values[0])
                        .placeholder(if variant == "inline-end" {
                            "Enter password"
                        } else {
                            "Search..."
                        })
                        .password(variant == "inline-end")
                        .show();
                    addon(
                        if variant == "inline-end" {
                            A::InlineEnd
                        } else {
                            A::InlineStart
                        },
                        || {
                            icon(if variant == "inline-end" {
                                I::EyeOff
                            } else {
                                I::Search
                            })
                        },
                    );
                });
                ui::field_description(if variant == "inline-end" {
                    "Icon positioned at the end."
                } else {
                    "Icon positioned at the start."
                });
            }),
            "block-end" => ui::column(28., || {
                ui::column(12., || {
                    ui::label("Input");
                    ui::InputGroup::new().disabled(disabled).show(|| {
                        ui::input_group_input(&mut state.input_group_values[0], "Enter amount");
                        addon(A::BlockEnd, || ui::input_group_text("USD"));
                    });
                    ui::field_description("Footer positioned below the input.");
                });
                ui::column(12., || {
                    ui::label("Textarea");
                    let mut post = false;
                    ui::InputGroup::new().disabled(disabled).show(|| {
                        ui::input_group_textarea(
                            &mut state.input_group_values[1],
                            "Write a comment...",
                        );
                        addon(A::BlockEnd, || {
                            ui::input_group_text(format!(
                                "{}/280",
                                state.input_group_values[1].chars().count()
                            ));
                            ui::spacer();
                            post = ui::input_group_button("Post")
                                .variant(ui::Variant::Default)
                                .size(ui::ButtonSize::Default)
                                .disabled(disabled)
                                .show()
                                .clicked;
                        });
                    });
                    if post {
                        state.input_group_values[1].clear();
                        state.count += 1;
                    }
                    ui::field_description("Footer positioned below the textarea.");
                });
            }),
            _ => unreachable!("unconfigured input-group preview"),
        },
    );
}
