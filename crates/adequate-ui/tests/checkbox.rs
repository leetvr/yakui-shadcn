//! Drive the actual field hit target and Yakui focus order.
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
fn helper(value: &mut bool) -> [ui::ControlResponse; 2] {
    let mut responses = [ui::ControlResponse::default(); 2];
    ui::column(8., || {
        ui::width(240., || {
            responses[0] = ui::Checkbox::new()
                .label("Receive updates")
                .description("Receive messages about your account and notifications.")
                .show(value);
        });
        responses[1] = ui::button("Continue");
    });
    responses
}
fn card_fields(values: &mut [bool; 2]) -> [ui::ControlResponse; 3] {
    let mut responses = [ui::ControlResponse::default(); 3];
    ui::column(8., || {
        ui::width(288., || {
            responses[0] = ui::Checkbox::new()
                .label("Enable notifications")
                .description("Change this at any time.")
                .card(true)
                .invalid(true)
                .show(&mut values[0]);
        });
        responses[1] = ui::Checkbox::new()
            .label("Disabled")
            .disabled(true)
            .show(&mut values[1]);
        responses[2] = ui::button("Next");
    });
    responses
}
fn key(y: &mut Yakui, key: KeyCode, down: bool) {
    y.handle_event(Event::KeyChanged {
        key,
        down,
        modifiers: None,
    });
}
fn click_at(y: &mut Yakui, position: Vec2) {
    y.handle_event(Event::CursorMoved(Some(position)));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
}
fn indicator_rect(y: &Yakui, id: WidgetId) -> Rect {
    let node = y.dom().get(id).unwrap();
    if node.widget.type_name().ends_with("::IndicatorRing") {
        return y.layout_dom().get(id).unwrap().rect;
    }
    node.children
        .iter()
        .find_map(|&child| {
            fn find(y: &Yakui, id: WidgetId) -> Option<Rect> {
                let node = y.dom().get(id).unwrap();
                if node.widget.type_name().ends_with("::IndicatorRing") {
                    return Some(y.layout_dom().get(id).unwrap().rect);
                }
                node.children.iter().find_map(|&child| find(y, child))
            }
            find(y, child)
        })
        .unwrap()
}
#[test]
fn checkbox_helper_text_activates_once_and_keeps_one_focus_stop() {
    let mut y = yak();
    let mut value = false;
    let responses = frame(&mut y, || helper(&mut value));
    let field = responses[0];
    let rect = y.layout_dom().get(field.id.unwrap()).unwrap().rect;
    let indicator = indicator_rect(&y, field.id.unwrap());
    assert!(rect.size().y > 40.);
    assert_eq!(indicator.size(), Vec2::splat(16.));
    assert_eq!(indicator.pos().y, rect.pos().y + 1.);
    click_at(&mut y, Vec2::new(rect.center().x, rect.max().y - 4.));
    assert!(frame(&mut y, || helper(&mut value))[0].clicked);
    assert!(value);
    assert!(!frame(&mut y, || helper(&mut value))[0].clicked);
    key(&mut y, KeyCode::Tab, true);
    key(&mut y, KeyCode::Tab, false);
    frame(&mut y, || helper(&mut value));
    assert!(frame(&mut y, || helper(&mut value))[1].focus_visible);
}

#[test]
fn checkbox_card_focus_activation_and_disabled_field_contract() {
    let mut y = yak();
    let mut values = [false; 2];
    frame(&mut y, || card_fields(&mut values));
    key(&mut y, KeyCode::Tab, true);
    key(&mut y, KeyCode::Tab, false);
    frame(&mut y, || card_fields(&mut values));
    frame(&mut y, || card_fields(&mut values));
    key(&mut y, KeyCode::Space, true);
    frame(&mut y, || card_fields(&mut values));
    assert!(!values[0]);
    key(&mut y, KeyCode::Space, false);
    frame(&mut y, || card_fields(&mut values));
    key(&mut y, KeyCode::Tab, true);
    key(&mut y, KeyCode::Tab, false);
    frame(&mut y, || card_fields(&mut values));
    let responses = frame(&mut y, || card_fields(&mut values));
    assert!(values[0]);
    assert!(!values[1]);
    assert!(responses[2].focus_visible);
    let disabled = y
        .layout_dom()
        .get(responses[1].id.unwrap())
        .unwrap()
        .rect
        .center();
    click_at(&mut y, disabled);
    frame(&mut y, || card_fields(&mut values));
    assert!(!values[1]);
}

#[test]
fn checkbox_bare_and_rtl_indicator_layout() {
    let mut y = yak();
    let mut values = [false; 2];
    let mut responses = [ui::ControlResponse::default(); 2];
    frame(&mut y, || {
        ui::column(8., || {
            responses[0] = ui::Checkbox::new().show(&mut values[0]);
            ui::direction(ui::ReadingDirection::Rtl, || {
                ui::width(240., || {
                    responses[1] = ui::Checkbox::new()
                        .label("Accept terms")
                        .description("A wrapped helper text field.")
                        .show(&mut values[1]);
                })
            });
        })
    });
    assert_eq!(
        y.layout_dom()
            .get(responses[0].id.unwrap())
            .unwrap()
            .rect
            .size(),
        Vec2::splat(16.)
    );
    let rect = y.layout_dom().get(responses[1].id.unwrap()).unwrap().rect;
    let indicator = indicator_rect(&y, responses[1].id.unwrap());
    assert_eq!(indicator.max().x, rect.max().x);
}
