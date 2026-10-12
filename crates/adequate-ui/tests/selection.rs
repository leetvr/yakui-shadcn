use yakui::{Vec2, Yakui, event::Event, input::KeyCode};
use yakui_adequate_ui as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    y.set_surface_size(Vec2::new(800., 600.));
    y.set_unscaled_viewport(yakui::Rect::from_pos_size(
        Vec2::ZERO,
        Vec2::new(800., 600.),
    ));
    ui::add_fonts(&mut y);
    y
}
fn frame(y: &mut Yakui, body: &mut impl FnMut()) {
    y.start();
    body();
    y.finish();
}
fn key(y: &mut Yakui, key: KeyCode) {
    y.handle_event(Event::KeyChanged {
        key,
        down: true,
        modifiers: None,
    });
    y.handle_event(Event::KeyChanged {
        key,
        down: false,
        modifiers: None,
    });
}
fn widgets(y: &Yakui, name: &str) -> Vec<yakui::WidgetId> {
    fn visit(y: &Yakui, id: yakui::WidgetId, name: &str, out: &mut Vec<yakui::WidgetId>) {
        let n = y.dom().get(id).unwrap();
        if n.widget.type_name().ends_with(name) {
            out.push(id);
        }
        for &child in &n.children {
            visit(y, child, name, out);
        }
    }
    let mut out = Vec::new();
    visit(y, y.dom().root(), name, &mut out);
    out
}
#[test]
fn select_arrows_and_enter_update_the_callers_value() {
    let mut y = yak();
    let mut state = ui::SelectState::default();
    let mut build = || {
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            ui::select(&mut state, &["Apple", "Banana", "Grapes"], "Select", false)
        });
        ui::select_content(&mut state, &["Apple", "Banana", "Grapes"]);
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Enter);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::Enter);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(state.selected, Some(1));
    assert!(!state.popup.open);
}
#[test]
fn menu_skips_disabled_and_toggles_caller_owned_checks() {
    let mut y = yak();
    let mut state = ui::MenuState::default();
    state.popup.open = true;
    let mut items = vec![
        ui::MenuItem::action(1, "First"),
        ui::MenuItem::action(2, "Disabled").disabled(true),
        ui::MenuItem::checkbox(3, "Bookmarks", false),
    ];
    let mut action = None;
    let mut build = || {
        action = ui::dropdown_menu(&mut state, &mut items);
    };
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::Enter);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(action, Some(3));
    assert!(matches!(items[2].kind, ui::MenuKind::Checkbox(true)));
}
#[test]
fn command_search_and_editor_arrow_down_activate_the_filtered_command() {
    let mut y = yak();
    let mut query = String::new();
    let items = vec![
        ui::MenuItem::action(1, "Profile"),
        ui::MenuItem::action(2, "Billing"),
    ];
    let mut action = None;
    let mut build = || {
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            action = ui::command(&mut query, &items);
        });
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    for c in "bill".chars() {
        y.handle_event(Event::TextInput(c));
    }
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::ArrowDown);
    key(&mut y, KeyCode::Enter);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(query, "bill");
    assert_eq!(action, Some(2));
}
#[test]
fn calendar_keyboard_crosses_month_without_selecting_until_enter() {
    let mut y = yak();
    let mut state = ui::CalendarState::new(ui::Date::new(2026, 10, 31).unwrap());
    let mut build = || {
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            ui::calendar(&mut state);
        });
    };
    frame(&mut y, &mut build);
    let days = widgets(&y, "::DayWidget");
    y.request_focus(Some(days[34]));
    key(&mut y, KeyCode::ArrowRight);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Enter);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(state.month, 11);
    assert_eq!(state.selected_date(), ui::Date::new(2026, 11, 1));
}

#[test]
fn radio_group_roves_one_tab_stop_and_selects_on_arrows() {
    let mut y = yak();
    let selected = std::cell::Cell::new(1);
    let after_focused = std::cell::Cell::new(false);
    let mut build = || {
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            ui::column(12., || {
                let mut value = selected.get();
                ui::radio_group(&["Default", "Comfortable", "Compact"], &mut value);
                selected.set(value);
                after_focused.set(ui::button("After").focused);
            });
        });
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut build);
    assert_eq!(selected.get(), 2);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut build);
    assert_eq!(selected.get(), 0);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    assert!(after_focused.get());
}

#[test]
fn radio_items_layout_in_separate_rows() {
    let mut y = yak();
    let mut selected = 1;
    let mut build = || {
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            ui::radio_group(&["Default", "Comfortable", "Compact"], &mut selected)
        });
    };
    frame(&mut y, &mut build);
    let items = widgets(&y, "::ChoiceWidget");
    let rects: Vec<_> = items
        .iter()
        .map(|id| y.layout_dom().get(*id).unwrap().rect)
        .collect();
    assert_eq!(rects.len(), 3);
    for pair in rects.windows(2) {
        assert_eq!(pair[1].pos().y - pair[0].pos().y, 24.);
    }
}
