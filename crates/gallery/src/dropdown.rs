//! Actual pinned Base UI dropdown previews, composed entirely from public APIs.
use ui::{Icon as I, MenuItem as M};
use yakui_shadcn as ui;
#[derive(Default)]
pub struct DropdownExample {
    variant: String,
    pub items: Vec<M>,
    notifications: [bool; 3],
}
pub const VARIANTS: &[&str] = &[
    "default",
    "basic",
    "submenu",
    "shortcuts",
    "icons",
    "checkboxes",
    "checkboxes-icons",
    "radio-group",
    "radio-icons",
    "destructive",
    "complex",
    "avatar",
];
fn account(shortcuts: bool) -> Vec<M> {
    let mut items = vec![
        M::label("My Account"),
        M::action(1, "Profile"),
        M::action(2, "Billing"),
        M::action(3, "Settings"),
    ];
    if shortcuts {
        for (item, shortcut) in items[1..].iter_mut().zip(["⇧⌘P", "⌘B", "⌘S"]) {
            item.shortcut = Some(shortcut.into());
        }
    }
    items
}
fn invite(nested: bool) -> M {
    let mut items = vec![M::action(21, "Email"), M::action(22, "Message")];
    if nested {
        items.push(M::submenu(
            23,
            "More options",
            vec![
                M::action(24, "Calendly"),
                M::action(25, "Slack"),
                M::separator(),
                M::action(26, "Webhook"),
            ],
        ));
    }
    items.push(M::separator());
    items.push(M::action(
        27,
        if nested { "Advanced..." } else { "More..." },
    ));
    M::submenu(20, "Invite users", items)
}
fn items(variant: &str) -> Vec<M> {
    match variant {
        "default" => {
            let mut v = account(true);
            v.extend([
                M::separator(),
                M::action(4, "Team"),
                invite(false),
                M::action(5, "New Team").shortcut("⌘+T"),
                M::separator(),
                M::action(6, "GitHub"),
                M::action(7, "Support"),
                M::action(8, "API").disabled(true),
                M::separator(),
                M::action(9, "Log out").shortcut("⇧⌘Q"),
            ]);
            v
        }
        "basic" => {
            let mut v = account(false);
            v.extend([
                M::separator(),
                M::action(6, "GitHub"),
                M::action(7, "Support"),
                M::action(8, "API").disabled(true),
            ]);
            v
        }
        "submenu" => vec![
            M::action(4, "Team"),
            invite(true),
            M::action(5, "New Team").shortcut("⌘+T"),
        ],
        "shortcuts" => {
            let mut v = account(true);
            v.extend([M::separator(), M::action(9, "Log out").shortcut("⇧⌘Q")]);
            v
        }
        "icons" => vec![
            M::action(1, "Profile").icon(I::User),
            M::action(2, "Billing").icon(I::CreditCard),
            M::action(3, "Settings").icon(I::Settings),
            M::separator(),
            M::action(9, "Log out").icon(I::LogOut).destructive(true),
        ],
        "checkboxes" => vec![
            M::label("Appearance"),
            M::checkbox(1, "Status Bar", true),
            M::checkbox(2, "Activity Bar", false).disabled(true),
            M::checkbox(3, "Panel", false),
        ],
        "checkboxes-icons" => vec![
            M::label("Notification Preferences"),
            M::checkbox(101, "Email notifications", true).icon(I::Mail),
            M::checkbox(102, "SMS notifications", false).icon(I::MessageSquare),
            M::checkbox(103, "Push notifications", true).icon(I::Bell),
        ],
        "radio-group" => vec![
            M::label("Panel Position"),
            M::radio(1, "Top", 1, false),
            M::radio(2, "Bottom", 1, true),
            M::radio(3, "Right", 1, false),
        ],
        "radio-icons" => vec![
            M::label("Select Payment Method"),
            M::radio(1, "Credit Card", 1, true).icon(I::CreditCard),
            M::radio(2, "PayPal", 1, false).icon(I::Wallet),
            M::radio(3, "Bank Transfer", 1, false).icon(I::Building2),
        ],
        "destructive" => vec![
            M::action(1, "Edit").icon(I::Pencil),
            M::action(2, "Share").icon(I::Share),
            M::separator(),
            M::action(3, "Delete").icon(I::Trash).destructive(true),
        ],
        "avatar" => vec![
            M::action(1, "Account").icon(I::BadgeCheck),
            M::action(2, "Billing").icon(I::CreditCard),
            M::action(3, "Notifications").icon(I::Bell),
            M::separator(),
            M::action(4, "Sign Out").icon(I::LogOut),
        ],
        "complex" => vec![
            M::label("File"),
            M::action(1, "New File").icon(I::File).shortcut("⌘N"),
            M::action(2, "New Folder").icon(I::Folder).shortcut("⇧⌘N"),
            M::submenu(
                3,
                "Open Recent",
                vec![
                    M::label("Recent Projects"),
                    M::action(4, "Project Alpha").icon(I::FileCode),
                    M::action(5, "Project Beta").icon(I::FileCode),
                    M::submenu(
                        6,
                        "More Projects",
                        vec![
                            M::action(7, "Project Gamma").icon(I::FileCode),
                            M::action(8, "Project Delta").icon(I::FileCode),
                        ],
                    )
                    .icon(I::MoreHorizontal),
                    M::separator(),
                    M::action(9, "Browse...").icon(I::FolderSearch),
                ],
            )
            .icon(I::FolderOpen),
            M::separator(),
            M::action(10, "Save").icon(I::Save).shortcut("⌘S"),
            M::action(11, "Export").icon(I::Download).shortcut("⇧⌘E"),
            M::separator(),
            M::label("View"),
            M::checkbox(101, "Show Sidebar", true).icon(I::Eye),
            M::checkbox(102, "Show Status Bar", false).icon(I::Layout),
            M::submenu(
                12,
                "Theme",
                vec![
                    M::label("Appearance"),
                    M::radio(13, "Light", 1, true).icon(I::Sun),
                    M::radio(14, "Dark", 1, false).icon(I::Moon),
                    M::radio(15, "System", 1, false).icon(I::Monitor),
                ],
            )
            .icon(I::Palette),
            M::separator(),
            M::label("Account"),
            M::action(16, "Profile").icon(I::User).shortcut("⇧⌘P"),
            M::action(17, "Billing").icon(I::CreditCard),
            M::submenu(
                18,
                "Settings",
                vec![
                    M::label("Preferences"),
                    M::action(19, "Keyboard Shortcuts").icon(I::Keyboard),
                    M::action(20, "Language").icon(I::Languages),
                    M::submenu(
                        21,
                        "Notifications",
                        vec![
                            M::label("Notification Types"),
                            M::checkbox(103, "Push Notifications", true).icon(I::Bell),
                            M::checkbox(101, "Email Notifications", true).icon(I::Mail),
                        ],
                    )
                    .icon(I::Bell),
                    M::separator(),
                    M::action(22, "Privacy & Security").icon(I::Shield),
                ],
            )
            .icon(I::Settings),
            M::separator(),
            M::action(23, "Help & Support").icon(I::HelpCircle),
            M::action(24, "Documentation").icon(I::FileText),
            M::separator(),
            M::action(25, "Sign Out")
                .icon(I::LogOut)
                .shortcut("⇧⌘Q")
                .destructive(true),
        ],
        _ => unreachable!("unconfigured dropdown preview"),
    }
}
pub fn trigger(
    variant: &str,
    example: &mut DropdownExample,
    menu: &mut ui::MenuState,
    disabled: bool,
) -> ui::ControlResponse {
    if example.variant != variant {
        example.items = items(variant);
        example.variant = variant.into();
        example.notifications = [true, false, true];
    }
    if variant == "avatar" {
        return ui::dropdown_menu_trigger(menu, || {
            ui::Button::new("")
                .variant(ui::Variant::Ghost)
                .size(ui::ButtonSize::Icon)
                .rounded(true)
                .disabled(disabled)
                .show_children(|| {
                    ui::Avatar::new("LR")
                        .image(crate::assets::avatar(0, false))
                        .show()
                })
        });
    }
    ui::dropdown_menu_trigger(menu, || {
        ui::Button::new(match variant {
            "checkboxes-icons" => "Notifications",
            "radio-icons" => "Payment Method",
            "destructive" => "Actions",
            "complex" => "Complex Menu",
            _ => "Open",
        })
        .variant(ui::Variant::Outline)
        .disabled(disabled)
        .show()
    })
}
fn synchronize(items: &mut [M], notifications: &[bool; 3]) {
    for item in items {
        if let ui::MenuKind::Checkbox(checked) = &mut item.kind {
            if (101..=103).contains(&item.id) {
                *checked = notifications[(item.id - 101) as usize];
            }
        }
        if let ui::MenuKind::Submenu(children) = &mut item.kind {
            synchronize(children, notifications);
        }
    }
}
pub fn content(
    variant: &str,
    example: &mut DropdownExample,
    menu: &mut ui::MenuState,
) -> Option<u64> {
    let width = match variant {
        "default" | "checkboxes" => 160.,
        "checkboxes-icons" => 192.,
        "radio-icons" => 0.,
        "complex" => 176.,
        _ => 0.,
    };
    let action = ui::dropdown_menu_with_options(
        menu,
        &mut example.items,
        ui::MenuOptions {
            width,
            align: if variant == "avatar" {
                ui::PopoverAlign::End
            } else {
                ui::PopoverAlign::Start
            },
            ..Default::default()
        },
    );
    if let Some(id @ 101..=103) = action {
        // The source's View and nested Notifications entries share one boolean.
        // Menu item copies report stable IDs; apply the mutation once to their
        // owner, then propagate that value to both visible compositions.
        example.notifications[(id - 101) as usize] = !example.notifications[(id - 101) as usize];
        synchronize(&mut example.items, &example.notifications);
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use yakui::{Alignment, Rect, Vec2, Yakui, event::Event, input::KeyCode};
    fn yak() -> Yakui {
        let mut y = Yakui::new();
        let size = Vec2::new(1000., 800.);
        y.set_surface_size(size);
        y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
        ui::add_fonts(&mut y);
        y
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
    fn frame(
        y: &mut Yakui,
        variant: &str,
        example: &mut DropdownExample,
        menu: &mut ui::MenuState,
    ) -> (ui::ControlResponse, Option<u64>) {
        y.start();
        let mut response = ui::ControlResponse::default();
        yakui::align(Alignment::CENTER, || {
            response = trigger(variant, example, menu, false)
        });
        let action = content(variant, example, menu);
        y.finish();
        (response, action)
    }
    fn check_values(items: &[M], id: u64, out: &mut Vec<bool>) {
        for item in items {
            if item.id == id {
                if let ui::MenuKind::Checkbox(value) = item.kind {
                    out.push(value);
                }
            }
            if let ui::MenuKind::Submenu(children) = &item.kind {
                check_values(children, id, out);
            }
        }
    }
    #[test]
    fn complex_dropdown_updates_shared_source_notifications_from_actual_menu_activation() {
        let mut y = yak();
        let mut menu = ui::MenuState::default();
        let mut example = DropdownExample::default();
        let (trigger, _) = frame(&mut y, "complex", &mut example, &mut menu);
        y.request_focus(trigger.id);
        key(&mut y, KeyCode::ArrowDown);
        for _ in 0..2 {
            frame(&mut y, "complex", &mut example, &mut menu);
        }
        // New File, New Folder, Open Recent, Save, Export, Show Sidebar.
        for _ in 0..5 {
            key(&mut y, KeyCode::ArrowDown);
            frame(&mut y, "complex", &mut example, &mut menu);
        }
        key(&mut y, KeyCode::Enter);
        assert_eq!(
            frame(&mut y, "complex", &mut example, &mut menu).1,
            Some(101)
        );
        assert!(menu.popup.open);
        let mut values = vec![];
        check_values(&example.items, 101, &mut values);
        assert_eq!(values, vec![false, false]);
        key(&mut y, KeyCode::Enter);
        frame(&mut y, "complex", &mut example, &mut menu);
        values.clear();
        check_values(&example.items, 101, &mut values);
        assert_eq!(values, vec![true, true]);
    }
    #[test]
    fn checkbox_dropdown_skips_disabled_activity_bar_and_retains_checked_state() {
        let mut y = yak();
        let mut menu = ui::MenuState::default();
        let mut example = DropdownExample::default();
        let (trigger, _) = frame(&mut y, "checkboxes", &mut example, &mut menu);
        y.request_focus(trigger.id);
        key(&mut y, KeyCode::ArrowDown);
        for _ in 0..2 {
            frame(&mut y, "checkboxes", &mut example, &mut menu);
        }
        key(&mut y, KeyCode::ArrowDown);
        key(&mut y, KeyCode::Enter);
        assert_eq!(
            frame(&mut y, "checkboxes", &mut example, &mut menu).1,
            Some(3)
        );
        assert!(menu.popup.open);
        assert!(matches!(
            example.items[1].kind,
            ui::MenuKind::Checkbox(true)
        ));
        assert!(matches!(
            example.items[2].kind,
            ui::MenuKind::Checkbox(false)
        ));
        assert!(matches!(
            example.items[3].kind,
            ui::MenuKind::Checkbox(true)
        ));
    }
}
