//! The pinned default navigation composition with app-owned route selection.
use ui::{NavigationMenuEntry as E, NavigationMenuLink as L};
use yakui_adequate_ui as ui;
pub fn render(state: &mut crate::examples::ExampleState, disabled: bool) {
    let entries = [
        E::trigger(1, "Getting started"),
        E::trigger(2, "Components"),
        E::trigger(3, "With Icon"),
        E::link(4, "Docs"),
    ]
    .map(|entry| entry.disabled(disabled));
    if let Some(route) = ui::navigation_menu_bar(&entries, &mut state.navigation) {
        state.count = route as u32;
    }
}
fn link(id: u64, title: &str, description: &str, clipped: bool) -> Option<u64> {
    let link = L::new(title).description(description);
    (if clipped {
        link.description_lines(2)
    } else {
        link
    })
    .show()
    .clicked
    .then_some(id)
}
pub fn content(state: &mut crate::examples::ExampleState) {
    let width = match state.navigation.active() {
        Some(2) => 508.,
        Some(3) => 208.,
        _ => 392.,
    };
    if let Some(action) = ui::navigation_menu_content(&mut state.navigation, width, |active| {
        let mut action = None;
        match active {
            1 => ui::column(0., || {
                for (id, title, description) in [
                    (
                        10,
                        "Introduction",
                        "Re-usable components built with Tailwind CSS.",
                    ),
                    (
                        11,
                        "Installation",
                        "How to install dependencies and structure your app.",
                    ),
                    (
                        12,
                        "Typography",
                        "Styles for headings, paragraphs, lists...etc",
                    ),
                ] {
                    if let Some(id) = link(id, title, description, false) {
                        action = Some(id);
                    }
                }
            }),
            2 => {
                let entries = [
                    (
                        20,
                        "Alert Dialog",
                        "A modal dialog that interrupts the user with important content and expects a response.",
                    ),
                    (
                        21,
                        "Hover Card",
                        "For sighted users to preview content available behind a link.",
                    ),
                    (
                        22,
                        "Progress",
                        "Displays an indicator showing the completion progress of a task, typically displayed as a progress bar.",
                    ),
                    (
                        23,
                        "Scroll-area",
                        "Visually or semantically separates content.",
                    ),
                    (
                        24,
                        "Tabs",
                        "A set of layered sections of content—known as tab panels—that are displayed one at a time.",
                    ),
                    (
                        25,
                        "Tooltip",
                        "A popup that displays information related to an element when the element receives keyboard focus or the mouse hovers over it.",
                    ),
                ];
                ui::column(8., || {
                    for row in 0..3 {
                        ui::row(8., || {
                            for column in 0..2 {
                                ui::width(246., || {
                                    let (id, title, description) = entries[row * 2 + column];
                                    if let Some(id) = link(id, title, description, true) {
                                        action = Some(id);
                                    }
                                });
                            }
                        });
                    }
                });
            }
            3 => ui::column(0., || {
                for (id, title, icon) in [
                    (30, "Backlog", ui::Icon::CircleAlert),
                    (31, "To Do", ui::Icon::CircleDashed),
                    (32, "Done", ui::Icon::CircleCheck),
                ] {
                    if L::new(title).icon(icon).show().clicked {
                        action = Some(id);
                    }
                }
            }),
            _ => {}
        }
        action
    }) {
        state.count = action as u32;
    }
}
