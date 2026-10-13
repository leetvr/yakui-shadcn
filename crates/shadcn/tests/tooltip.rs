//! A pointer-only tooltip must not acquire keyboard focus as it closes.
use yakui::{Rect, Vec2, Yakui, event::Event};
use yakui_shadcn as ui;
fn frame(y: &mut Yakui) -> ui::ControlResponse {
    y.start();
    let mut response = ui::ControlResponse::default();
    yakui::align(yakui::Alignment::TOP_LEFT, || {
        response = ui::button("Hover");
    });
    ui::tooltip(response, "Useful information");
    y.finish();
    response
}
#[test]
fn pointer_tooltip_closing_does_not_focus_its_trigger_and_reopen_itself() {
    let mut y = Yakui::new();
    let size = Vec2::new(600., 400.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    let response = frame(&mut y);
    let rect = y.layout_dom().get(response.id.unwrap()).unwrap().rect;
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    assert!(frame(&mut y).hovering);
    frame(&mut y);
    y.handle_event(Event::CursorMoved(Some(Vec2::new(500., 350.))));
    frame(&mut y);
    frame(&mut y);
    let response = frame(&mut y);
    assert!(!response.hovering);
    assert!(!response.focused);
}
fn rich_frame(
    y: &mut Yakui,
    state: &mut ui::TooltipState,
    side: ui::Side,
    area: bool,
) -> ui::ControlResponse {
    y.start();
    let mut trigger = ui::ControlResponse::default();
    yakui::align(yakui::Alignment::TOP_LEFT, || {
        yakui::pad(
            yakui::widgets::Pad {
                left: 250.,
                top: 180.,
                right: 0.,
                bottom: 0.,
            },
            || {
                trigger = if area {
                    ui::tooltip_trigger_area(state, || {
                        ui::Button::new("Disabled").disabled(true).show();
                    })
                } else {
                    ui::tooltip_trigger(state, || ui::button("Hover"))
                };
            },
        );
    });
    ui::Tooltip::new().side(side).show(state, "Add to library");
    y.finish();
    trigger
}
fn key(y: &mut Yakui, key: yakui::input::KeyCode) {
    for down in [true, false] {
        y.handle_event(Event::KeyChanged {
            key,
            down,
            modifiers: None,
        });
    }
}
fn tooltip_rect(y: &Yakui, id: yakui::WidgetId) -> Option<Rect> {
    let node = y.dom().get(id)?;
    if node.widget.type_name().ends_with("tooltip::Arrow") {
        return y.layout_dom().get(id).map(|node| node.rect);
    }
    node.children.iter().find_map(|id| tooltip_rect(y, *id))
}
#[test]
fn tooltip_places_natural_content_on_each_side_without_moving_focus() {
    let mut y = Yakui::new();
    let size = Vec2::new(600., 400.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    let mut state = ui::TooltipState::default();
    for side in [
        ui::Side::Top,
        ui::Side::Bottom,
        ui::Side::Left,
        ui::Side::Right,
    ] {
        let trigger = rich_frame(&mut y, &mut state, side, false);
        let anchor = y.layout_dom().get(trigger.id.unwrap()).unwrap().rect;
        y.handle_event(Event::CursorMoved(Some(anchor.center())));
        rich_frame(&mut y, &mut state, side, false);
        rich_frame(&mut y, &mut state, side, false);
        let popup = tooltip_rect(&y, y.dom().root()).unwrap();
        assert!(popup.size().x < 140. && popup.size().x > 70.);
        assert_eq!(popup.size().y, 28.);
        match side {
            ui::Side::Top => assert_eq!(popup.max().y, anchor.pos().y - 4.),
            ui::Side::Bottom => assert_eq!(popup.pos().y, anchor.max().y + 4.),
            ui::Side::Left => assert_eq!(popup.max().x, anchor.pos().x - 4.),
            ui::Side::Right => assert_eq!(popup.pos().x, anchor.max().x + 4.),
        }
        assert!(!rich_frame(&mut y, &mut state, side, false).focused);
    }
}
#[test]
fn focused_tooltip_escape_suppresses_reopening_and_disabled_area_is_hover_only() {
    let mut y = Yakui::new();
    let size = Vec2::new(600., 400.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    let mut state = ui::TooltipState::default();
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    key(&mut y, yakui::input::KeyCode::Tab);
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    assert!(state.is_open());
    key(&mut y, yakui::input::KeyCode::Escape);
    let trigger = rich_frame(&mut y, &mut state, ui::Side::Top, false);
    assert!(trigger.focused);
    assert!(!state.is_open());
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    assert!(!state.is_open());
    y.handle_event(Event::RequestFocus(None));
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    key(&mut y, yakui::input::KeyCode::Tab);
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    rich_frame(&mut y, &mut state, ui::Side::Top, false);
    assert!(state.is_open());
    y.handle_event(Event::RequestFocus(None));
    let area = rich_frame(&mut y, &mut state, ui::Side::Top, true);
    let rect = y.layout_dom().get(area.id.unwrap()).unwrap().rect;
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    rich_frame(&mut y, &mut state, ui::Side::Top, true);
    assert!(state.is_open());
    key(&mut y, yakui::input::KeyCode::Tab);
    let area = rich_frame(&mut y, &mut state, ui::Side::Top, true);
    assert!(!area.focused && !area.clicked);
}
