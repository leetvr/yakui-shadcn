//! Gallery compositions contain only public Adequate components and application state.
use ui::{Button, ButtonSize as S, Variant as V};
use yakui_adequate_ui as ui;

pub const AVAILABLE: &[&str] = &[
    "alert",
    "avatar",
    "badge",
    "button",
    "button-group",
    "card",
    "checkbox",
    "empty",
    "field",
    "input",
    "kbd",
    "label",
    "progress",
    "radio-group",
    "scroll-area",
    "separator",
    "skeleton",
    "spinner",
    "switch",
    "textarea",
    "toggle",
    "toggle-group",
    "typography",
    "accordion",
    "alert-dialog",
    "breadcrumb",
    "collapsible",
    "dialog",
    "drawer",
    "hover-card",
    "pagination",
    "popover",
    "sheet",
    "tabs",
    "tooltip",
];
#[derive(Default)]
pub struct ExampleState {
    pub text: String,
    pub checked: bool,
    pub selected: usize,
    pub toggle: bool,
    pub count: u32,
    pub overlay: ui::OverlayState,
    pub trigger: ui::ControlResponse,
    pub second: String,
}
/// Render the actual example. Unknown variants are rejected at the CLI boundary.
pub fn render(component: &str, variant: &str, state: &mut ExampleState, disabled: bool) {
    match component {
        "dialog" | "alert-dialog" | "drawer" | "sheet" | "popover" | "hover-card" | "tooltip" => {
            state.trigger = Button::new(match component {
                "dialog" => "Edit Profile",
                "alert-dialog" => "Show Dialog",
                "drawer" => "Open Drawer",
                "sheet" => "Open",
                "popover" => "Open popover",
                _ => "Hover",
            })
            .variant(V::Outline)
            .show();
            if state.trigger.clicked {
                state.overlay.toggle_from(state.trigger);
            }
            if component == "hover-card" {
                state.overlay.open = state.trigger.hovering || state.trigger.focused;
                state.overlay.trigger = state.trigger.id;
            }
        }
        "tabs" => ui::width(400., || {
            ui::tabs(
                &["Account", "Password"],
                &mut state.selected,
                if variant == "line" {
                    ui::TabsVariant::Line
                } else {
                    ui::TabsVariant::Default
                },
                |selected| {
                    ui::card(|| {
                        ui::card_header(|| {
                            ui::card_title(if selected == 0 { "Account" } else { "Password" });
                            ui::card_description("Make changes to your account here.");
                        });
                        ui::card_content(|| {
                            ui::text("Click save when you’re done.");
                        });
                        ui::card_footer(|| {
                            ui::button("Save changes");
                        });
                    })
                },
            )
        }),
        "collapsible" => ui::collapsible(
            "@peduarte starred 3 repositories",
            &mut state.checked,
            || {
                ui::text("@radix-ui/primitives");
                ui::text("@radix-ui/colors");
                ui::text("@stitches/react");
            },
        ),
        "accordion" => ui::accordion_item("Is it accessible?", &mut state.checked, 400., || {
            ui::text("Yes. It adheres to the WAI-ARIA design pattern.")
        }),
        "breadcrumb" => {
            ui::breadcrumb(&["Home", "Components", "Breadcrumb"]);
        }
        "pagination" => ui::pagination(&mut state.selected, 5),
        "button" => match variant {
            "demo" => ui::row(8., || {
                Button::new("Button").show();
                Button::new("Outline").variant(V::Outline).show();
                Button::new("")
                    .variant(V::Outline)
                    .size(S::Icon)
                    .icon(ui::Icon::Plus)
                    .show();
            }),
            "size" => ui::column(16., || {
                ui::row(8., || {
                    for (size, title) in [
                        (S::Xs, "Extra Small"),
                        (S::Sm, "Small"),
                        (S::Default, "Default"),
                        (S::Lg, "Large"),
                    ] {
                        Button::new(title).size(size).show();
                    }
                });
                ui::row(8., || {
                    for size in [S::IconXs, S::IconSm, S::Icon, S::IconLg] {
                        Button::new("").icon(ui::Icon::Plus).size(size).show();
                    }
                });
            }),
            "icon" => {
                Button::new("")
                    .variant(V::Outline)
                    .size(S::Icon)
                    .icon(ui::Icon::Plus)
                    .show();
            }
            "with-icon" => {
                Button::new("Send email").icon(ui::Icon::Mail).show();
            }
            "rounded" => {
                Button::new("Button").rounded(true).show();
            }
            "spinner" => {
                ui::row(8., || {
                    ui::spinner(16., 0.);
                    Button::new("Please wait").disabled(true).show();
                });
            }
            other => {
                let (variant, title) = match other {
                    "outline" => (V::Outline, "Outline"),
                    "secondary" => (V::Secondary, "Secondary"),
                    "ghost" => (V::Ghost, "Ghost"),
                    "destructive" => (V::Destructive, "Destructive"),
                    "link" => (V::Link, "Link"),
                    _ => (V::Default, "Button"),
                };
                if Button::new(title)
                    .variant(variant)
                    .disabled(disabled)
                    .show()
                    .clicked
                {
                    state.count += 1;
                }
            }
        },
        "badge" => {
            ui::row(8., || {
                for v in [
                    V::Default,
                    V::Secondary,
                    V::Destructive,
                    V::Outline,
                    V::Ghost,
                    V::Link,
                ] {
                    ui::badge("Badge", v);
                }
            });
        }
        "card" => ui::width(350., || {
            ui::card(|| {
                ui::card_header(|| {
                    ui::card_title("Create project");
                    ui::card_description("Deploy your new project in one-click.");
                });
                ui::card_content(|| {
                    ui::column(16., || {
                        ui::field("Name", "The name of your project.", None, || {
                            ui::input(&mut state.text, "Name of your project");
                        });
                        ui::field("Framework", "Choose your framework.", None, || {
                            ui::radio_group(
                                &["Next.js", "SvelteKit", "Astro"],
                                &mut state.selected,
                            );
                        });
                    })
                });
                ui::card_footer(|| {
                    ui::row(16., || {
                        Button::new("Cancel").variant(V::Outline).show();
                        ui::button("Deploy");
                    })
                });
            })
        }),
        "input" => ui::width(280., || {
            ui::input(&mut state.text, "Email");
        }),
        "textarea" => ui::width(320., || {
            ui::textarea(&mut state.text, "Type your message here.");
        }),
        "checkbox" => {
            ui::checkbox_disabled("Accept terms and conditions", &mut state.checked, disabled);
        }
        "switch" => {
            ui::switch("Airplane Mode", &mut state.checked, disabled);
        }
        "toggle" => {
            ui::toggle("Bold", &mut state.toggle, variant == "outline");
        }
        "toggle-group" => ui::row(0., || {
            for (i, title) in ["Bold", "Italic", "Underline"].iter().enumerate() {
                let mut selected = state.selected == i;
                if ui::toggle(*title, &mut selected, false).clicked {
                    state.selected = i;
                }
            }
        }),
        "radio-group" => {
            ui::radio_group(&["Default", "Comfortable", "Compact"], &mut state.selected)
        }
        "label" => {
            ui::label("Your email address");
        }
        "alert" => ui::width(450., || {
            ui::alert(
                "Heads up!",
                "You can add components to your app using the CLI.",
                variant == "destructive",
            )
        }),
        "avatar" => ui::avatar("CN", 32.),
        "progress" => ui::progress(0.6, 320.),
        "separator" => ui::width(320., || {
            ui::column(16., || {
                ui::label("Radix Primitives");
                ui::muted("An open-source UI component library.");
                ui::separator(320.);
                ui::row(16., || {
                    ui::text("Blog");
                    ui::text("Docs");
                    ui::text("Source");
                });
            })
        }),
        "skeleton" => ui::row(16., || {
            ui::skeleton(ui::Vec2::splat(48.));
            ui::column(8., || {
                ui::skeleton(ui::Vec2::new(250., 16.));
                ui::skeleton(ui::Vec2::new(200., 16.));
            });
        }),
        "spinner" => ui::spinner(16., 0.),
        "kbd" => ui::row(4., || {
            ui::kbd("⌘");
            ui::kbd("K");
        }),
        "field" => ui::width(320., || {
            ui::field(
                "Email",
                "We’ll never share your email.",
                if variant == "invalid" {
                    Some("Enter a valid email address.")
                } else {
                    None
                },
                || {
                    ui::input(&mut state.text, "you@example.com");
                },
            )
        }),
        "empty" => ui::empty(
            "No projects yet",
            "Get started by creating your first project.",
            || {
                ui::button("Create project");
            },
        ),
        "button-group" => ui::button_group(|| {
            for title in ["Archive", "Report", "Delete"] {
                Button::new(title).variant(V::Outline).show();
            }
        }),
        "scroll-area" => ui::width(200., || {
            ui::scroll_area(280., || {
                ui::column(12., || {
                    ui::label("Tags");
                    for i in 1..=50 {
                        ui::text(format!("v1.2.0-beta.{i}"));
                        ui::separator(180.);
                    }
                })
            })
        }),
        "typography" => ui::width(450., || {
            ui::column(16., || {
                ui::heading(1, "The Joke Tax Chronicles");
                ui::muted("Once upon a time, in a far-off land...");
                ui::heading(2, "The People of the Kingdom");
                ui::text(
                    "The king, seeing how much happier his subjects were, decided to reverse the tax.",
                );
                ui::heading(3, "The Joke Tax");
                ui::heading(4, "People stopped telling jokes");
            })
        }),
        _ => ui::empty(
            "Implementation pending",
            "See reference/components.json for the complete upstream scope.",
            || {},
        ),
    }
}
pub fn variants(component: &str) -> &'static [&'static str] {
    match component {
        "button" => &[
            "default",
            "demo",
            "size",
            "outline",
            "secondary",
            "ghost",
            "destructive",
            "link",
            "icon",
            "with-icon",
            "rounded",
            "spinner",
        ],
        "alert" => &["default", "destructive"],
        "tabs" => &["default", "line"],
        "sheet" => &["default", "left", "top", "bottom"],
        "toggle" => &["default", "outline"],
        "field" => &["default", "invalid"],
        _ => &["default"],
    }
}

/// Overlay content is declared at the root, outside the gallery scroll areas.
pub fn overlays(component: &str, variant: &str, state: &mut ExampleState) {
    let title = if component == "alert-dialog" {
        "Are you absolutely sure?"
    } else {
        "Edit profile"
    };
    let description = if component == "alert-dialog" {
        "This action cannot be undone. This will permanently delete your account."
    } else {
        "Make changes to your profile here. Click save when you're done."
    };
    let text = &mut state.text;
    let second = &mut state.second;
    let body = |overlay: &mut ui::OverlayState| {
        ui::card_title(title);
        ui::muted(description);
        if component != "alert-dialog" {
            ui::field("Name", "", None, || {
                ui::input(text, "Pedro Duarte");
            });
            ui::field("Username", "", None, || {
                ui::input(second, "@peduarte");
            });
        }
        ui::row(8., || {
            if Button::new("Cancel").variant(V::Outline).show().clicked {
                overlay.close();
            }
            if ui::button(if component == "alert-dialog" {
                "Continue"
            } else {
                "Save changes"
            })
            .clicked
            {
                overlay.close();
            }
        });
    };
    match component {
        "dialog" => ui::dialog(&mut state.overlay, body),
        "alert-dialog" => ui::alert_dialog(&mut state.overlay, body),
        "sheet" => ui::sheet(
            &mut state.overlay,
            match variant {
                "left" => ui::Side::Left,
                "top" => ui::Side::Top,
                "bottom" => ui::Side::Bottom,
                _ => ui::Side::Right,
            },
            body,
        ),
        "drawer" => ui::drawer(&mut state.overlay, body),
        "popover" => ui::popover(&mut state.overlay, 320., body),
        "hover-card" => ui::hover_card(&mut state.overlay, 320., |_| {
            ui::label("@nextjs");
            ui::text("The React Framework – created and maintained by @vercel.");
        }),
        "tooltip" => ui::tooltip(state.trigger, "Add to library"),
        _ => {}
    }
}
