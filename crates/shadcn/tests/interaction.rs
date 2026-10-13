//! Real Yakui event/layout cycles. No renderer, mocks, or fixture DOMs.
use yakui::{
    Vec2, Yakui,
    event::Event,
    input::{KeyCode, MouseButton},
};
use yakui_shadcn as ui;
fn apply() -> ui::ControlResponse {
    ui::button("Apply")
}
fn accept(value: &mut bool) -> ui::ControlResponse {
    ui::checkbox("Accept", value)
}
fn disabled() -> ui::ControlResponse {
    ui::Button::new("Apply").disabled(true).show()
}
fn yak() -> Yakui {
    let mut y = Yakui::new();
    y.set_surface_size(Vec2::new(600., 400.));
    y.set_unscaled_viewport(yakui::Rect::from_pos_size(
        Vec2::ZERO,
        Vec2::new(600., 400.),
    ));
    ui::add_fonts(&mut y);
    y
}
fn frame<T>(yak: &mut Yakui, body: impl FnOnce() -> T) -> T {
    yak.start();
    let mut res = None;
    yakui::align(yakui::Alignment::TOP_LEFT, || res = Some(body()));
    yak.finish();
    res.unwrap()
}
fn key(yak: &mut Yakui, key: KeyCode, down: bool) {
    yak.handle_event(Event::KeyChanged {
        key,
        down,
        modifiers: None,
    });
}
fn click(yak: &mut Yakui, id: yakui::WidgetId) {
    let rect = yak.layout_dom().get(id).unwrap().rect;
    yak.handle_event(Event::CursorMoved(Some(rect.center())));
    yak.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: true,
    });
    yak.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: false,
    });
}
#[test]
fn pointer_activation_is_consumed_once() {
    let mut y = yak();
    let first = frame(&mut y, apply);
    click(&mut y, first.id.unwrap());
    assert!(frame(&mut y, apply).clicked);
    assert!(!frame(&mut y, apply).clicked);
}
#[test]
fn releasing_outside_cancels_activation() {
    let mut y = yak();
    let first = frame(&mut y, apply);
    let rect = y.layout_dom().get(first.id.unwrap()).unwrap().rect;
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: true,
    });
    y.handle_event(Event::CursorMoved(Some(Vec2::new(599., 399.))));
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: false,
    });
    assert!(!frame(&mut y, apply).clicked);
}
#[test]
fn keyboard_enter_and_space_have_distinct_activation_edges() {
    let mut y = yak();
    let b = frame(&mut y, apply);
    y.request_focus(b.id);
    key(&mut y, KeyCode::Space, true);
    assert!(!frame(&mut y, apply).clicked);
    key(&mut y, KeyCode::Space, false);
    assert!(frame(&mut y, apply).clicked);
    key(&mut y, KeyCode::Enter, true);
    assert!(frame(&mut y, apply).clicked);
    key(&mut y, KeyCode::Enter, true);
    assert!(!frame(&mut y, apply).clicked);
    key(&mut y, KeyCode::Enter, false);
}
#[test]
fn losing_focus_cancels_a_pending_space() {
    let mut y = yak();
    let b = frame(&mut y, apply);
    y.request_focus(b.id);
    key(&mut y, KeyCode::Space, true);
    y.request_focus(None);
    key(&mut y, KeyCode::Space, false);
    assert!(!frame(&mut y, apply).clicked);
}
#[test]
fn disabled_controls_ignore_pointer_and_keyboard() {
    let mut y = yak();
    let b = frame(&mut y, disabled);
    click(&mut y, b.id.unwrap());
    y.request_focus(b.id);
    key(&mut y, KeyCode::Enter, true);
    assert!(!frame(&mut y, disabled).clicked);
}
#[test]
fn tab_skips_disabled_buttons() {
    let mut y = yak();
    let mut responses = [ui::ControlResponse::default(); 3];
    let mut build = || {
        ui::row(8., || {
            for (i, disabled) in [false, true, false].iter().enumerate() {
                responses[i] = ui::Button::new("Button").disabled(*disabled).show();
            }
        });
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab, true);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab, false);
    key(&mut y, KeyCode::Tab, true);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    drop(build);
    assert!(!responses[0].focused);
    assert!(!responses[1].focused);
    assert!(responses[2].focused);
}
#[test]
fn checkbox_updates_caller_owned_state() {
    let mut y = yak();
    let mut checked = false;
    let b = frame(&mut y, || accept(&mut checked));
    click(&mut y, b.id.unwrap());
    frame(&mut y, || accept(&mut checked));
    assert!(checked);
}
#[test]
fn textbox_edits_unicode_in_application_state() {
    let mut y = yak();
    let mut text = String::new();
    frame(&mut y, || {
        ui::width(200., || {
            ui::input(&mut text, "Name");
        })
    });
    key(&mut y, KeyCode::Tab, true);
    frame(&mut y, || {
        ui::width(200., || {
            ui::input(&mut text, "Name");
        })
    });
    y.handle_event(Event::TextInput('猫'));
    frame(&mut y, || {
        ui::width(200., || {
            ui::input(&mut text, "Name");
        })
    });
    assert_eq!(text, "猫");
    key(&mut y, KeyCode::Backspace, true);
    frame(&mut y, || {
        ui::width(200., || {
            ui::input(&mut text, "Name");
        })
    });
    assert_eq!(text, "");
}
#[test]
fn sizes_match_pinned_nova_heights() {
    let mut y = yak();
    let mut out = Vec::new();
    frame(&mut y, || {
        ui::row(8., || {
            for s in [
                ui::ButtonSize::Xs,
                ui::ButtonSize::Sm,
                ui::ButtonSize::Default,
                ui::ButtonSize::Lg,
            ] {
                out.push(ui::Button::new("Button").size(s).show());
            }
        })
    });
    let sizes: Vec<_> = out
        .iter()
        .map(|r| y.layout_dom().get(r.id.unwrap()).unwrap().rect.size().y)
        .collect();
    assert_eq!(sizes, [24., 28., 32., 36.]);
}
#[test]
fn nested_themes_are_scoped_and_restore_the_parent() {
    let mut y = yak();
    frame(&mut y, || {
        ui::with_theme(ui::Theme::dark(), || {
            assert!(ui::theme().dark);
            ui::with_theme(ui::Theme::light(), || assert!(!ui::theme().dark));
            assert!(ui::theme().dark);
        })
    });
}

#[test]
fn buttons_keep_natural_width_and_center_when_explicitly_stretched() {
    let mut y = yak();
    let mut buttons = Vec::new();
    frame(&mut y, || {
        ui::column(8., || {
            buttons.push(ui::button("Apply"));
            ui::width(300., || {
                ui::fill_width(|| {
                    buttons.push(ui::button("Apply"));
                })
            });
        })
    });
    let rects: Vec<_> = buttons
        .iter()
        .map(|r| y.layout_dom().get(r.id.unwrap()).unwrap().rect)
        .collect();
    assert!(rects[0].size().x < 100.);
    assert_eq!(rects[1].size().x, 300.);
    fn text_rect(y: &Yakui, id: yakui::WidgetId) -> Option<yakui::Rect> {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("::RenderTextWidget") {
            return Some(y.layout_dom().get(id).unwrap().rect);
        }
        node.children.iter().find_map(|id| text_rect(y, *id))
    }
    let text = text_rect(&y, buttons[1].id.unwrap()).unwrap();
    assert!((text.center().x - rects[1].center().x).abs() < 0.01);
}
