//! Native route callbacks, reachable sibling triggers and keyboard dismissal.
use yakui::{
    Alignment, Rect, Vec2, WidgetId, Yakui,
    event::Event,
    input::{KeyCode, MouseButton},
};
use yakui_shadcn as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    let size = Vec2::new(600., 400.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    y
}
fn frame(y: &mut Yakui, state: &mut ui::NavigationMenuState) -> (Option<u64>, Option<u64>) {
    y.start();
    let mut route = None;
    yakui::align(Alignment::TOP_LEFT, || {
        route = ui::navigation_menu_bar(
            &[
                ui::NavigationMenuEntry::trigger(1, "First"),
                ui::NavigationMenuEntry::trigger(2, "Second"),
                ui::NavigationMenuEntry::link(3, "Docs"),
            ],
            state,
        );
    });
    let action = ui::navigation_menu_content(state, 240., |_| {
        ui::NavigationMenuLink::new("Open project")
            .description("Use the current project.")
            .show()
            .clicked
            .then_some(42)
    });
    y.finish();
    (route, action)
}
fn controls(y: &Yakui) -> Vec<WidgetId> {
    fn visit(y: &Yakui, id: WidgetId, ids: &mut Vec<WidgetId>) {
        let n = y.dom().get(id).unwrap();
        if n.widget.type_name().ends_with("::ControlWidget") {
            ids.push(id);
        }
        for &c in &n.children {
            visit(y, c, ids);
        }
    }
    let mut ids = vec![];
    visit(y, y.dom().root(), &mut ids);
    ids
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
fn key(y: &mut Yakui, key: KeyCode) {
    for down in [true, false] {
        y.handle_event(Event::KeyChanged {
            key,
            down,
            modifiers: None,
        });
    }
}
#[test]
fn navigation_flyout_keeps_sibling_triggers_reachable_and_returns_link_ids_once() {
    let mut y = yak();
    let mut state = ui::NavigationMenuState::default();
    frame(&mut y, &mut state);
    let triggers = controls(&y);
    click(&mut y, triggers[0]);
    frame(&mut y, &mut state);
    assert_eq!(state.active(), Some(1));
    click(&mut y, triggers[1]);
    frame(&mut y, &mut state);
    assert_eq!(state.active(), Some(2));
    let action = *controls(&y).last().unwrap();
    click(&mut y, action);
    assert_eq!(frame(&mut y, &mut state).1, Some(42));
    assert_eq!(state.active(), None);
    assert_eq!(frame(&mut y, &mut state).1, None);
    frame(&mut y, &mut state);
    click(&mut y, triggers[2]);
    assert_eq!(frame(&mut y, &mut state).0, Some(3));
    assert_eq!(state.active(), None);
}
#[test]
fn navigation_keyboard_open_escape_and_outside_click_close_the_actual_popup() {
    let mut y = yak();
    let mut state = ui::NavigationMenuState::default();
    frame(&mut y, &mut state);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut state);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut state);
    frame(&mut y, &mut state);
    assert_eq!(state.active(), Some(1));
    key(&mut y, KeyCode::Escape);
    frame(&mut y, &mut state);
    frame(&mut y, &mut state);
    assert_eq!(state.active(), None);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut state);
    assert_eq!(state.active(), Some(1));
    y.handle_event(Event::CursorMoved(Some(Vec2::new(500., 350.))));
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: true,
    });
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: false,
    });
    frame(&mut y, &mut state);
    assert_eq!(state.active(), None);
}
