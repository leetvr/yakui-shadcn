//! Native switch state, focus and exact documented track sizes.
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
fn frame<T>(y: &mut Yakui, body: impl FnOnce() -> T) -> T {
    y.start();
    let mut result = None;
    yakui::align(Alignment::TOP_LEFT, || result = Some(body()));
    y.finish();
    result.unwrap()
}
fn key(y: &mut Yakui, key: KeyCode, down: bool) {
    y.handle_event(Event::KeyChanged {
        key,
        down,
        modifiers: None,
    });
}
fn indicator(y: &Yakui, id: WidgetId) -> Rect {
    let node = y.dom().get(id).unwrap();
    fn find(y: &Yakui, id: WidgetId) -> Option<Rect> {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("::IndicatorRing") {
            return Some(y.layout_dom().get(id).unwrap().rect);
        }
        node.children.iter().find_map(|&child| find(y, child))
    }
    node.children
        .iter()
        .find_map(|&child| find(y, child))
        .unwrap()
}
fn field(value: &mut bool) -> [ui::ControlResponse; 2] {
    let mut responses = [ui::ControlResponse::default(); 2];
    ui::column(8., || {
        ui::width(288., || {
            responses[0] = ui::Switch::new()
                .label("Share across devices")
                .description("Focus is shared across devices and turns off when you leave the app.")
                .trailing(true)
                .invalid(true)
                .show(value);
        });
        responses[1] = ui::button("Next");
    });
    responses
}
#[test]
fn switch_trailing_helper_text_activates_the_caller_value_and_has_one_focus_stop() {
    let mut y = yak();
    let mut value = false;
    let responses = frame(&mut y, || field(&mut value));
    let rect = y.layout_dom().get(responses[0].id.unwrap()).unwrap().rect;
    let track = indicator(&y, responses[0].id.unwrap());
    assert_eq!(track.max().x, rect.max().x);
    assert_eq!(track.pos().y, rect.pos().y);
    y.handle_event(Event::CursorMoved(Some(Vec2::new(
        rect.pos().x + 10.,
        rect.max().y - 4.,
    ))));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
    assert!(frame(&mut y, || field(&mut value))[0].clicked);
    assert!(value);
    key(&mut y, KeyCode::Space, true);
    assert!(!frame(&mut y, || field(&mut value))[0].clicked);
    key(&mut y, KeyCode::Space, false);
    assert!(frame(&mut y, || field(&mut value))[0].clicked);
    assert!(!value);
    key(&mut y, KeyCode::Tab, true);
    key(&mut y, KeyCode::Tab, false);
    frame(&mut y, || field(&mut value));
    assert!(frame(&mut y, || field(&mut value))[1].focus_visible);
}
#[test]
fn switch_tracks_have_documented_sizes_and_disabled_switches_skip_focus() {
    let mut y = yak();
    let mut values = [false; 2];
    fn sizes(values: &mut [bool; 2]) -> [ui::ControlResponse; 3] {
        let mut responses = [ui::ControlResponse::default(); 3];
        ui::column(8., || {
            responses[0] = ui::Switch::new()
                .size(ui::SwitchSize::Sm)
                .disabled(true)
                .show(&mut values[0]);
            responses[1] = ui::Switch::new().show(&mut values[1]);
            responses[2] = ui::button("Next");
        });
        responses
    }
    frame(&mut y, || sizes(&mut values));
    key(&mut y, KeyCode::Tab, true);
    key(&mut y, KeyCode::Tab, false);
    frame(&mut y, || sizes(&mut values));
    let responses = frame(&mut y, || sizes(&mut values));
    assert!(!responses[0].focused);
    assert!(responses[1].focus_visible);
    assert_eq!(
        y.layout_dom()
            .get(responses[0].id.unwrap())
            .unwrap()
            .rect
            .size(),
        Vec2::new(24., 14.)
    );
    assert_eq!(
        y.layout_dom()
            .get(responses[1].id.unwrap())
            .unwrap()
            .rect
            .size(),
        Vec2::new(32., 18.4)
    );
    let disabled = y
        .layout_dom()
        .get(responses[0].id.unwrap())
        .unwrap()
        .rect
        .center();
    y.handle_event(Event::CursorMoved(Some(disabled)));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
    frame(&mut y, || sizes(&mut values));
    assert_eq!(values, [false; 2]);
}
