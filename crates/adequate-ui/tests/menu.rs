//! Drive the real trigger, popup focus, keyboard selection and geometry.
use yakui::{
    Alignment, Rect, Vec2, WidgetId, Yakui,
    event::Event,
    input::{KeyCode, MouseButton},
};
use yakui_adequate_ui as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    y.set_surface_size(Vec2::new(600., 400.));
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, Vec2::new(600., 400.)));
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
fn click(y: &mut Yakui, id: WidgetId) {
    let rect = y.layout_dom().get(id).unwrap().rect;
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
}
fn frame(
    y: &mut Yakui,
    state: &mut ui::MenuState,
    items: &mut [ui::MenuItem],
) -> (ui::ControlResponse, Option<u64>) {
    y.start();
    let mut trigger = ui::ControlResponse::default();
    yakui::align(Alignment::TOP_RIGHT, || {
        yakui::pad(yakui::widgets::Pad::all(20.), || {
            trigger = ui::dropdown_menu_trigger(state, || {
                ui::Button::new("Open").variant(ui::Variant::Outline).show()
            });
        });
    });
    let action = ui::dropdown_menu_with_options(
        state,
        items,
        ui::MenuOptions {
            width: 128.,
            align: ui::PopoverAlign::End,
            ..Default::default()
        },
    );
    y.finish();
    (trigger, action)
}
fn menu_rows(y: &Yakui) -> Vec<Rect> {
    fn visit(y: &Yakui, id: WidgetId, out: &mut Vec<Rect>) {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("::EntryWidget") {
            out.push(y.layout_dom().get(id).unwrap().rect);
        }
        for &child in &node.children {
            visit(y, child, out);
        }
    }
    let mut rows = Vec::new();
    visit(y, y.dom().root(), &mut rows);
    rows
}
#[test]
fn pointer_open_focuses_popup_without_selecting_a_row_and_down_enters_first_item() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items = [
        ui::MenuItem::action(1, "Edit"),
        ui::MenuItem::action(2, "Disabled").disabled(true),
        ui::MenuItem::action(3, "Delete").destructive(true),
    ];
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    click(&mut y, trigger.id.unwrap());
    frame(&mut y, &mut state, &mut items);
    frame(&mut y, &mut state, &mut items);
    assert!(state.popup.open);
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, None);
    assert!(state.popup.open);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(1));
    assert!(!state.popup.open);
    frame(&mut y, &mut state, &mut items);
    assert!(frame(&mut y, &mut state, &mut items).0.focused);
}
#[test]
fn keyboard_up_opens_at_last_enabled_item_and_popup_aligns_to_trigger_end() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items = [
        ui::MenuItem::action(1, "Edit"),
        ui::MenuItem::action(2, "Delete"),
        ui::MenuItem::action(3, "Disabled").disabled(true),
    ];
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    y.request_focus(trigger.id);
    key(&mut y, KeyCode::ArrowUp);
    frame(&mut y, &mut state, &mut items);
    frame(&mut y, &mut state, &mut items);
    let rows = menu_rows(&y);
    assert_eq!(rows.len(), 3);
    let trigger_rect = y.layout_dom().get(trigger.id.unwrap()).unwrap().rect;
    assert!((rows[0].max().x + 4. - trigger_rect.max().x).abs() < 0.01);
    assert!((rows[0].pos().y - trigger_rect.max().y - 8.).abs() < 0.01);
    assert_eq!(rows[0].size(), Vec2::new(120., 28.));
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(2));
}
#[test]
fn popup_pointer_focus_escape_and_typeahead_skip_disabled_items() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items = [
        ui::MenuItem::action(1, "Disabled").disabled(true),
        ui::MenuItem::action(2, "Duplicate"),
        ui::MenuItem::action(3, "Edit"),
    ];
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    click(&mut y, trigger.id.unwrap());
    frame(&mut y, &mut state, &mut items);
    y.handle_event(Event::TextInput('d'));
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(2));
    frame(&mut y, &mut state, &mut items);
    click(&mut y, trigger.id.unwrap());
    frame(&mut y, &mut state, &mut items);
    key(&mut y, KeyCode::Escape);
    frame(&mut y, &mut state, &mut items);
    assert!(!state.popup.open);
}

#[test]
fn checkbox_and_radio_items_keep_the_popup_open_and_write_selection() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items = [
        ui::MenuItem::checkbox(1, "Show Sidebar", false),
        ui::MenuItem::radio(2, "Top", 10, false),
        ui::MenuItem::radio(3, "Bottom", 10, true),
    ];
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    click(&mut y, trigger.id.unwrap());
    frame(&mut y, &mut state, &mut items);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(1));
    assert!(matches!(items[0].kind, ui::MenuKind::Checkbox(true)));
    assert!(state.popup.open);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(2));
    assert!(matches!(
        items[1].kind,
        ui::MenuKind::Radio { selected: true, .. }
    ));
    assert!(matches!(
        items[2].kind,
        ui::MenuKind::Radio {
            selected: false,
            ..
        }
    ));
    assert!(state.popup.open);
}

#[test]
fn long_menu_keyboard_navigation_keeps_the_highlighted_row_in_view() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items: Vec<_> = (0..30)
        .map(|i| ui::MenuItem::action(i, format!("Command {i}")))
        .collect();
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    click(&mut y, trigger.id.unwrap());
    frame(&mut y, &mut state, &mut items);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::End);
    frame(&mut y, &mut state, &mut items);
    frame(&mut y, &mut state, &mut items);
    assert!(menu_rows(&y)[29].max().y <= 392.);
    key(&mut y, KeyCode::ArrowUp);
    frame(&mut y, &mut state, &mut items);
    frame(&mut y, &mut state, &mut items);
    let previous = menu_rows(&y)[28];
    assert!(previous.pos().y >= 8. && previous.max().y <= 392.);
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(28));
}

#[test]
fn nested_menu_measures_its_own_width_and_returns_deep_action_then_restores_root() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items = vec![ui::MenuItem::submenu(
        1,
        "Projects",
        vec![
            ui::MenuItem::action(2, "A long project name"),
            ui::MenuItem::submenu(
                3,
                "More options",
                vec![
                    ui::MenuItem::action(4, "First"),
                    ui::MenuItem::action(5, "Last"),
                ],
            ),
        ],
    )];
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    y.request_focus(trigger.id);
    key(&mut y, KeyCode::ArrowDown);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    key(&mut y, KeyCode::ArrowRight);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    let rows = menu_rows(&y);
    assert_eq!(rows.len(), 3);
    assert!(rows[1].size().x > 128.);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::ArrowRight);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    assert_eq!(menu_rows(&y).len(), 5);
    key(&mut y, KeyCode::End);
    key(&mut y, KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(5));
    assert!(!state.popup.open);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    assert!(frame(&mut y, &mut state, &mut items).0.focused);
}

#[test]
fn submenu_escape_returns_to_parent_and_pointer_can_activate_another_root_item() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items = vec![
        ui::MenuItem::submenu(1, "Projects", vec![ui::MenuItem::action(2, "Alpha")]),
        ui::MenuItem::action(3, "Root action"),
    ];
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    y.request_focus(trigger.id);
    key(&mut y, KeyCode::ArrowDown);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    key(&mut y, KeyCode::ArrowRight);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    assert_eq!(menu_rows(&y).len(), 3);
    key(&mut y, KeyCode::Escape);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    assert!(state.popup.open);
    assert_eq!(menu_rows(&y).len(), 2);
    key(&mut y, KeyCode::ArrowRight);
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    assert_eq!(menu_rows(&y).len(), 3);
    let row = menu_rows(&y)[1];
    y.handle_event(Event::CursorMoved(Some(row.center())));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
    let result = frame(&mut y, &mut state, &mut items).1;
    assert_eq!(result, Some(3));
    assert!(!state.popup.open);
}

#[test]
fn narrow_menu_wraps_text_between_leading_icon_and_trailing_check() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items =
        vec![ui::MenuItem::radio(1, "Bank Transfer", 1, true).icon(ui::Icon::Building2)];
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    click(&mut y, trigger.id.unwrap());
    for _ in 0..2 {
        frame(&mut y, &mut state, &mut items);
    }
    let row = menu_rows(&y)[0];
    assert_eq!(row.size(), Vec2::new(120., 48.));
}

#[test]
fn bottom_anchored_menu_flips_and_pointer_activation_is_consumed_once() {
    fn frame(
        y: &mut Yakui,
        state: &mut ui::MenuState,
        items: &mut [ui::MenuItem],
    ) -> (ui::ControlResponse, Option<u64>) {
        y.start();
        let mut trigger = ui::ControlResponse::default();
        yakui::align(Alignment::BOTTOM_RIGHT, || {
            yakui::pad(yakui::widgets::Pad::all(20.), || {
                trigger = ui::dropdown_menu_trigger(state, || ui::button("Open"));
            });
        });
        let action = ui::dropdown_menu(state, items);
        y.finish();
        (trigger, action)
    }
    let mut y = yak();
    let mut state = ui::MenuState::default();
    let mut items: Vec<_> = (0..10)
        .map(|id| ui::MenuItem::action(id, format!("Command {id}")))
        .collect();
    let (trigger, _) = frame(&mut y, &mut state, &mut items);
    click(&mut y, trigger.id.unwrap());
    for _ in 0..3 {
        frame(&mut y, &mut state, &mut items);
    }
    let rows = menu_rows(&y);
    let trigger = y.layout_dom().get(trigger.id.unwrap()).unwrap().rect;
    assert!(rows[9].max().y < trigger.pos().y);
    y.handle_event(Event::CursorMoved(Some(rows[4].center())));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
    assert_eq!(frame(&mut y, &mut state, &mut items).1, Some(4));
    assert_eq!(frame(&mut y, &mut state, &mut items).1, None);
    assert!(!state.popup.open);
}
