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
    "calendar",
    "date-picker",
    "table",
    "data-table",
    "select",
    "native-select",
    "combobox",
    "command",
    "dropdown-menu",
    "context-menu",
    "menubar",
    "navigation-menu",
    "aspect-ratio",
    "attachment",
    "bubble",
    "chart",
    "carousel",
    "direction",
    "form",
    "input-group",
    "input-otp",
    "item",
    "marker",
    "message",
    "message-scroller",
    "questionnaire",
    "resizable",
    "sidebar",
    "slider",
    "toast",
];
pub struct ExampleState {
    pub text: String,
    pub checked: bool,
    pub selected: usize,
    pub radio: usize,
    pub toggle: bool,
    pub count: u32,
    pub phase: f32,
    pub overlay: ui::OverlayState,
    pub trigger: ui::ControlResponse,
    pub second: String,
    pub calendar: ui::CalendarState,
    pub data_table: ui::DataTableState,
    pub select: ui::SelectState,
    pub menu: ui::MenuState,
    pub menubar: ui::MenubarState,
    pub menu_items: Vec<ui::MenuItem>,
    pub values: [f64; 2],
    pub ratio: f32,
    pub carousel: ui::CarouselState,
    pub sidebar: ui::SidebarState,
    pub questionnaire: ui::QuestionnaireState,
    pub scroller: ui::MessageScrollerState,
    pub chat: Vec<String>,
    pub attachment: ui::Attachment,
    pub toasts: ui::ToastManager,
    pub chart: ui::ChartResponse,
    pub toggles: ui::ToggleGroupState,
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
        "calendar" => {
            ui::calendar(&mut state.calendar);
        }
        "date-picker" => ui::date_picker(&mut state.calendar, &mut state.overlay),
        "table" | "data-table" => {
            let columns = [
                ui::TableColumn::new("Status", 100.),
                ui::TableColumn::new("Email", 240.),
                ui::TableColumn::new("Amount", 100.).numeric(),
            ];
            let rows = vec![
                ui::TableRow::new(1, ["success", "m@example.com", "$316.00"]),
                ui::TableRow::new(2, ["success", "p@example.com", "$242.00"]),
                ui::TableRow::new(3, ["processing", "ken99@example.com", "$837.00"]),
                ui::TableRow::new(4, ["success", "abe45@example.com", "$874.00"]),
                ui::TableRow::new(5, ["failed", "carmella@example.com", "$721.00"]),
            ];
            if component == "table" {
                ui::table(&columns, &rows, Some("A list of your recent invoices."));
            } else {
                ui::data_table(&columns, &rows, &mut state.data_table);
            }
        }
        "select" | "native-select" => ui::select(
            &mut state.select,
            &["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"],
            "Select a fruit",
            disabled,
        ),
        "combobox" => ui::combobox(
            &mut state.select,
            &["Next.js", "SvelteKit", "Nuxt", "Remix", "Astro"],
            "Select framework...",
        ),
        "command" => {
            if let Some(id) = ui::command(&mut state.text, &state.menu_items) {
                state.count = id as u32;
            }
        }
        "dropdown-menu" => {
            state.trigger = Button::new("Open").variant(V::Outline).show();
            if state.trigger.clicked {
                state.menu.open_from(state.trigger);
            }
        }
        "context-menu" => ui::context_menu_target(&mut state.menu, || {
            ui::surface(
                ui::Color::CLEAR,
                Some(ui::theme().border),
                ui::theme().radius,
                48.,
                || ui::text("Right click here"),
            )
        }),
        "menubar" => ui::menubar(&["File", "Edit", "View", "Profile"], &mut state.menubar),
        "navigation-menu" => {
            ui::navigation_menu(
                &["Getting started", "Components", "Documentation"],
                &mut state.selected,
            );
        }
        "aspect-ratio" => ui::aspect_ratio(16. / 9., 450., || {
            ui::surface(ui::theme().muted, None, ui::theme().radius, 0., || {
                yakui::center(|| ui::text("16 : 9"));
            })
        }),
        "attachment" => {
            let response = ui::attachment(&state.attachment);
            if response.removed {
                state.attachment.state = ui::AttachmentState::Idle;
            }
            if response.retry {
                state.attachment.state = ui::AttachmentState::Uploading;
            }
            if response.opened {
                state.count += 1;
            }
        }
        "bubble" => ui::width(380., || {
            ui::column(12., || {
                ui::bubble(ui::BubbleVariant::Muted, || {
                    ui::text("Does your demo show how to build a useful chat interface?")
                });
                ui::bubble(ui::BubbleVariant::Default, || {
                    ui::text("Sure. Hit me with your best demo.")
                });
            })
        }),
        "chart" => {
            let source: Vec<serde_json::Value> =
                serde_json::from_str(include_str!("../../../reference/chart-demo.json")).unwrap();
            ui::column(16., || {
                ui::card_title("Bar Chart - Interactive");
                ui::card_description("Showing total visitors for the last 3 months");
                ui::row(8., || {
                    for (i, label) in ["Desktop", "Mobile"].iter().enumerate() {
                        if Button::new(*label)
                            .variant(if state.selected == i {
                                V::Secondary
                            } else {
                                V::Ghost
                            })
                            .show()
                            .clicked
                        {
                            state.selected = i;
                        }
                    }
                });
                let points = source
                    .iter()
                    .map(|p| {
                        ui::ChartPoint::new(
                            p["date"].as_str().unwrap(),
                            vec![
                                p[if state.selected == 0 {
                                    "desktop"
                                } else {
                                    "mobile"
                                }]
                                .as_f64()
                                .unwrap(),
                            ],
                        )
                    })
                    .collect();
                state.chart = ui::Chart::new(
                    points,
                    vec![ui::ChartSeries::new(
                        if state.selected == 0 {
                            "Desktop"
                        } else {
                            "Mobile"
                        },
                        ui::Color::hex(0x2563eb),
                    )],
                )
                .kind(match variant {
                    "line" => ui::ChartKind::Line,
                    "area" => ui::ChartKind::Area,
                    "pie" => ui::ChartKind::Pie,
                    "donut" => ui::ChartKind::Donut,
                    "radar" => ui::ChartKind::Radar,
                    "radial" => ui::ChartKind::Radial,
                    "scatter" => ui::ChartKind::Scatter,
                    _ => ui::ChartKind::Bar,
                })
                .show();
            });
        }
        "carousel" => ui::carousel(
            &mut state.carousel,
            5,
            ui::CarouselOptions {
                orientation: if variant == "vertical" {
                    ui::Orientation::Vertical
                } else {
                    ui::Orientation::Horizontal
                },
                ..Default::default()
            },
            |i| {
                ui::card(|| {
                    ui::card_content(|| {
                        yakui::center(|| ui::heading(1, (i + 1).to_string()));
                    })
                })
            },
        ),
        "direction" => ui::direction(ui::ReadingDirection::Rtl, || {
            ui::breadcrumb(&["Home", "Components", "Direction"])
                .map(|_| ())
                .unwrap_or(())
        }),
        "form" => ui::width(360., || {
            ui::form(|| {
                ui::field(
                    "Username",
                    "This is your public display name.",
                    None,
                    || {
                        ui::input(&mut state.text, "shadcn");
                    },
                );
                ui::checkbox("Receive email updates", &mut state.checked);
                if ui::button("Submit").clicked {
                    state.count += 1;
                }
            })
        }),
        "input-group" => ui::input_group(|| {
            ui::icon(ui::Icon::Search, 16.);
            ui::width(240., || {
                ui::input(&mut state.text, "Search...");
            });
            ui::kbd("⌘ K");
        }),
        "input-otp" => {
            ui::input_otp(
                &mut state.text,
                ui::OtpOptions {
                    disabled,
                    ..Default::default()
                },
            );
        }
        "item" => ui::item(V::Outline, || {
            ui::row(16., || {
                ui::avatar("CN", 32.);
                ui::column(4., || {
                    ui::label("Basic Item");
                    ui::muted("A simple item with a title and description.");
                });
                ui::button("Action");
            })
        }),
        "marker" => ui::marker("Today", ui::MarkerVariant::Separator, 400.),
        "message" => ui::width(420., || {
            ui::message("Alex", "AL", "9:41 AM", ui::MessageAlign::Start, || {
                ui::bubble(ui::BubbleVariant::Muted, || {
                    ui::text("Hey! Can you help me build a chat interface?")
                })
            })
        }),
        "message-scroller" => ui::width(440., || {
            ui::column(12., || {
                ui::message_scroller(ui::Vec2::new(440., 300.), &mut state.scroller, || {
                    for (i, message) in state.chat.iter().enumerate() {
                        ui::message_scroller_item(i as u64, || {
                            ui::message(
                                if i % 2 == 0 { "You" } else { "Assistant" },
                                if i % 2 == 0 { "YO" } else { "AI" },
                                "9:41 AM",
                                if i % 2 == 0 {
                                    ui::MessageAlign::End
                                } else {
                                    ui::MessageAlign::Start
                                },
                                || {
                                    ui::bubble(
                                        if i % 2 == 0 {
                                            ui::BubbleVariant::Default
                                        } else {
                                            ui::BubbleVariant::Muted
                                        },
                                        || ui::width(340., || ui::text(message)),
                                    )
                                },
                            )
                        });
                    }
                });
                ui::input(&mut state.text, "Type a message...");
                if ui::button("Send message").clicked && !state.text.trim().is_empty() {
                    state.chat.push(std::mem::take(&mut state.text));
                }
            })
        }),
        "questionnaire" => ui::width(440., || {
            let questions = questions();
            if ui::questionnaire(&questions, &mut state.questionnaire).submitted {
                state.count += 1;
            }
        }),
        "resizable" => ui::resizable(
            ui::Vec2::new(400., 200.),
            ui::Orientation::Horizontal,
            &mut state.ratio,
            || {
                yakui::center(|| ui::label("One"));
            },
            || {
                yakui::center(|| ui::label("Two"));
            },
        ),
        "sidebar" => ui::sidebar(
            ui::Vec2::new(640., 360.),
            &mut state.sidebar,
            "Acme Inc.",
            &[
                ui::SidebarLink::new("Home", ui::Icon::Menu),
                ui::SidebarLink::new("Inbox", ui::Icon::Mail),
                ui::SidebarLink::new("Settings", ui::Icon::Settings),
            ],
            |active| {
                ui::column(16., || {
                    ui::heading(3, ["Home", "Inbox", "Settings"][active]);
                    ui::card(|| {
                        ui::card_header(|| ui::card_title("Your workspace"));
                        ui::card_content(|| ui::text("Use the sidebar to navigate."));
                    });
                })
            },
        ),
        "slider" => {
            if variant == "range" {
                ui::range_slider(&mut state.values, ui::SliderOptions::default());
            } else {
                ui::slider(
                    &mut state.values[0],
                    ui::SliderOptions {
                        orientation: if variant == "vertical" {
                            ui::Orientation::Vertical
                        } else {
                            ui::Orientation::Horizontal
                        },
                        disabled,
                        ..Default::default()
                    },
                );
            }
        }
        "toast" => {
            if Button::new("Show Toast").variant(V::Outline).show().clicked {
                let id = state.toasts.add(
                    "Event created",
                    "Sunday, December 3 at 9:00 AM",
                    ui::ToastKind::Default,
                );
                if let Some(toast) = state.toasts.toasts.iter_mut().find(|t| t.id == id) {
                    toast.action = Some("Undo".into());
                }
            }
        }
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
                    ui::spinner(16., state.phase);
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
                for (v, title) in [
                    (V::Default, "Badge"),
                    (V::Secondary, "Secondary"),
                    (V::Destructive, "Destructive"),
                    (V::Outline, "Outline"),
                ] {
                    ui::badge(title, v);
                }
            });
        }
        "card" => ui::width(384., || {
            ui::card(|| {
                ui::card_header_with_action(
                    || {
                        ui::card_title("Login to your account");
                        ui::card_description("Enter your email below to login to your account");
                    },
                    || {
                        if Button::new("Sign Up").variant(V::Link).show().clicked {
                            state.count += 1;
                        }
                    },
                );
                ui::card_content(|| {
                    ui::column(24., || {
                        ui::field("Email", "", None, || {
                            ui::input(&mut state.text, "m@example.com");
                        });
                        ui::column(8., || {
                            ui::row(48., || {
                                ui::label("Password");
                                Button::new("Forgot your password?").variant(V::Link).show();
                            });
                            ui::fill_width(|| {
                                ui::Input::new(&mut state.second).password(true).show();
                            });
                        });
                    })
                });
                ui::card_footer(|| {
                    ui::column(8., || {
                        ui::fill_width(|| {
                            if ui::button("Login").clicked {
                                state.count += 1;
                            }
                        });
                        ui::fill_width(|| {
                            if Button::new("Login with Google")
                                .variant(V::Outline)
                                .show()
                                .clicked
                            {
                                state.count += 1;
                            }
                        });
                    })
                });
            })
        }),
        "input" => ui::width(if variant == "basic" { 384. } else { 280. }, || {
            ui::Input::new(&mut state.text)
                .placeholder(if variant == "basic" {
                    "Enter text"
                } else {
                    "Email"
                })
                .disabled(disabled)
                .invalid(variant == "invalid")
                .show();
        }),
        "textarea" => ui::width(384., || {
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
        "toggle-group" => {
            ui::toggle_group(&["Bold", "Italic", "Underline"], &mut state.toggles, false)
        }
        "radio-group" => ui::radio_group(&["Default", "Comfortable", "Compact"], &mut state.radio),
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
        "progress" => ui::progress(0.66, 360.),
        "separator" => ui::width(384., || {
            ui::column(16., || {
                ui::column(6., || {
                    ui::label("shadcn/ui");
                    ui::muted("The Foundation for your Design System");
                });
                ui::separator(384.);
                ui::text(
                    "A set of beautifully designed components that you can customize, extend, and build on.",
                );
            })
        }),
        "skeleton" => ui::row(16., || {
            ui::skeleton_at(ui::Vec2::splat(48.), 999., state.phase);
            ui::column(8., || {
                ui::skeleton_at(
                    ui::Vec2::new(250., 16.),
                    ui::theme().radius * 0.8,
                    state.phase,
                );
                ui::skeleton_at(
                    ui::Vec2::new(200., 16.),
                    ui::theme().radius * 0.8,
                    state.phase,
                );
            });
        }),
        "spinner" => ui::spinner(16., state.phase),
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
        "chart" => &[
            "default", "line", "area", "pie", "donut", "radar", "radial", "scatter",
        ],
        "slider" => &["default", "range", "vertical"],
        "carousel" => &["default", "vertical"],
        "sheet" => &["default", "left", "top", "bottom"],
        "toggle" => &["default", "outline"],
        "field" => &["default", "invalid"],
        "input" => &["default", "basic", "invalid"],
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
        "toast" => {
            let response = ui::toast_viewport(&mut state.toasts);
            if let Some(id) = response.action {
                state.toasts.close(id);
            }
        }
        "chart" => {
            if let Some(index) = state.chart.hovered {
                ui::tooltip(
                    ui::ControlResponse {
                        hovering: true,
                        id: state.chart.id,
                        ..Default::default()
                    },
                    {
                        let data: Vec<serde_json::Value> = serde_json::from_str(include_str!(
                            "../../../reference/chart-demo.json"
                        ))
                        .unwrap();
                        let point = &data[index];
                        let key = if state.selected == 0 {
                            "desktop"
                        } else {
                            "mobile"
                        };
                        format!(
                            "{}\n{}: {}",
                            point["date"].as_str().unwrap(),
                            key,
                            point[key]
                        )
                    },
                );
            }
        }
        "date-picker" => ui::date_picker_content(&mut state.calendar, &mut state.overlay),
        "select" | "native-select" => ui::select_content(
            &mut state.select,
            &["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"],
        ),
        "combobox" => ui::combobox_content(
            &mut state.select,
            &["Next.js", "SvelteKit", "Nuxt", "Remix", "Astro"],
        ),
        "dropdown-menu" | "context-menu" => {
            if let Some(id) = ui::dropdown_menu(&mut state.menu, &mut state.menu_items) {
                state.count = id as u32;
            }
        }
        "menubar" => {
            if let Some(id) = ui::dropdown_menu(&mut state.menubar.menu, &mut state.menu_items) {
                state.count = id as u32;
            }
        }
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

impl ExampleState {
    pub fn is_open(&self, component: &str) -> bool {
        match component {
            "select" | "native-select" | "combobox" => self.select.popup.open,
            "dropdown-menu" | "context-menu" => self.menu.popup.open,
            "menubar" => self.menubar.menu.popup.open,
            "tooltip" => self.trigger.hovering || self.trigger.focused,
            _ => self.overlay.open,
        }
    }
}

impl Default for ExampleState {
    fn default() -> Self {
        Self {
            text: String::new(),
            second: String::new(),
            checked: false,
            selected: 0,
            radio: 1,
            toggle: false,
            count: 0,
            phase: 0.,
            overlay: Default::default(),
            trigger: Default::default(),
            calendar: ui::CalendarState::new(ui::Date::new(2026, 10, 9).unwrap()),
            data_table: Default::default(),
            select: Default::default(),
            menu: Default::default(),
            menubar: Default::default(),
            values:[75.,90.],ratio:0.5,carousel:Default::default(),sidebar:Default::default(),questionnaire:Default::default(),scroller:Default::default(),
            chat:vec!["I'm building a chat for our app and the scroll behavior is driving me nuts.".into(),"Wrap your message list in MessageScroller and turn on auto-follow. The viewport pins to the bottom as text arrives.".into(),"And if they've scrolled up to re-read an older answer?".into(),"Auto-follow stops when the reader scrolls up. Their position is preserved as new messages arrive.".into()],
            attachment:ui::Attachment::new("message-renderer.tsx","TypeScript · 12 KB"),toasts:Default::default(),chart:Default::default(),toggles:Default::default(),
            menu_items: vec![
                ui::MenuItem::action(1, "Profile"),
                ui::MenuItem::action(2, "Billing"),
                ui::MenuItem::action(3, "Settings"),
                ui::MenuItem::separator(),
                ui::MenuItem::checkbox(4, "Show bookmarks", true),
                ui::MenuItem::submenu(
                    5,
                    "Invite users",
                    vec![
                        ui::MenuItem::action(6, "Email"),
                        ui::MenuItem::action(7, "Message"),
                    ],
                ),
                ui::MenuItem::action(8, "Log out"),
            ],
        }
    }
}

fn questions() -> Vec<ui::Question> {
    vec![
        ui::Question {
            name: "direction".into(),
            title: "What should the agent build next?".into(),
            description: "Choose a direction or describe another task.".into(),
            choices: vec![
                ui::QuestionChoice {
                    label: "Tool call timeline".into(),
                    description: "Show what the agent ran and what came back.".into(),
                },
                ui::QuestionChoice {
                    label: "Approval checkpoints".into(),
                    description: "Ask before sensitive or destructive actions.".into(),
                },
                ui::QuestionChoice {
                    label: "Sub-agent handoffs".into(),
                    description: "Make delegated work and results easier to follow.".into(),
                },
            ],
            required: true,
            multiple: false,
            allow_other: true,
        },
        ui::Question {
            name: "signals".into(),
            title: "What should every progress update include?".into(),
            description: "Select all that apply, or skip this question.".into(),
            choices: ["Progress", "Decisions", "Risks", "Next step"]
                .iter()
                .map(|label| ui::QuestionChoice {
                    label: (*label).into(),
                    description: String::new(),
                })
                .collect(),
            required: false,
            multiple: true,
            allow_other: false,
        },
        ui::Question {
            name: "timing".into(),
            title: "When should work begin?".into(),
            description: "Choose when the agent should begin the work.".into(),
            choices: [
                "Start now",
                "Next development cycle",
                "Add it to the backlog",
            ]
            .iter()
            .map(|label| ui::QuestionChoice {
                label: (*label).into(),
                description: String::new(),
            })
            .collect(),
            required: true,
            multiple: false,
            allow_other: false,
        },
    ]
}
