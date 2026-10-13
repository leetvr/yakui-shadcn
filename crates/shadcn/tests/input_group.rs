//! Real layout and input cycles for the common search/composer mechanisms.
use yakui::{Alignment, Rect, Vec2, Yakui, event::Event, input::MouseButton};
use yakui_shadcn as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    let size = Vec2::new(600., 400.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    y
}
fn frame(
    y: &mut Yakui,
    text: &mut String,
    disabled: bool,
) -> (ui::InputResponse, ui::ControlResponse) {
    y.start();
    let mut input = ui::InputResponse::default();
    let mut action = ui::ControlResponse::default();
    yakui::align(Alignment::TOP_LEFT, || {
        ui::width(320., || {
            ui::InputGroup::new().disabled(disabled).show(|| {
                // Deliberately declare the editor first, matching the upstream demo.
                input = ui::input_group_input(text, "Search...");
                ui::input_group_addon(ui::InputGroupAlign::InlineEnd, || {
                    action = ui::input_group_button("Search").show();
                });
                ui::input_group_addon(ui::InputGroupAlign::InlineStart, || {
                    ui::icon(ui::Icon::Search, 16.)
                });
            })
        })
    });
    y.finish();
    (input, action)
}
fn click(y: &mut Yakui, position: Vec2) {
    y.handle_event(Event::CursorMoved(Some(position)));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
}
#[test]
fn input_group_reorders_addons_budgets_editor_width_and_keeps_action_hits() {
    let mut y = yak();
    let mut text = String::new();
    let (input, button) = frame(&mut y, &mut text, false);
    let editor = y.layout_dom().get(input.id.unwrap()).unwrap().rect;
    let action = y.layout_dom().get(button.id.unwrap()).unwrap().rect;
    assert_eq!(editor.pos().x, 24.);
    assert!((editor.max().x - action.pos().x).abs() < 0.01);
    assert!(editor.size().x > 200.);
    click(&mut y, Vec2::new(12., 16.));
    frame(&mut y, &mut text, false);
    y.handle_event(Event::TextInput('q'));
    frame(&mut y, &mut text, false);
    assert_eq!(text, "q");
    click(&mut y, action.center());
    let (_, response) = frame(&mut y, &mut text, false);
    assert!(response.clicked);
    assert!(!frame(&mut y, &mut text, false).1.clicked);
    frame(&mut y, &mut text, true);
    click(&mut y, Vec2::new(12., 16.));
    frame(&mut y, &mut text, true);
    y.handle_event(Event::TextInput('x'));
    frame(&mut y, &mut text, true);
    assert_eq!(text, "q");
}
fn block_frame(y: &mut Yakui, text: &mut String) -> Option<yakui::WidgetId> {
    let mut id = None;
    y.start();
    yakui::align(Alignment::TOP_LEFT, || {
        ui::width(320., || {
            ui::input_group(|| {
                id = ui::input_group_textarea(text, "Write a message...").id;
                ui::input_group_addon(ui::InputGroupAlign::BlockEnd, || {
                    ui::input_group_text("120 characters left")
                });
                ui::input_group_addon(ui::InputGroupAlign::BlockStart, || {
                    ui::input_group_text("New message")
                });
            })
        })
    });
    y.finish();
    id
}
#[test]
fn input_group_block_addons_surround_a_full_width_multiline_editor() {
    let mut y = yak();
    let mut text = String::new();
    let id = block_frame(&mut y, &mut text);
    let rect = y.layout_dom().get(id.unwrap()).unwrap().rect;
    assert_eq!(rect.size().x, 320.);
    assert_eq!(rect.pos().y, 36.);
    assert!(rect.size().y >= 64.);
    y.request_focus(id);
    for c in "Hello".chars() {
        y.handle_event(Event::TextInput(c));
    }
    y.handle_event(Event::KeyChanged {
        key: yakui::input::KeyCode::Enter,
        modifiers: Default::default(),
        down: true,
    });
    y.handle_event(Event::KeyChanged {
        key: yakui::input::KeyCode::Enter,
        modifiers: Default::default(),
        down: false,
    });
    for c in "world".chars() {
        y.handle_event(Event::TextInput(c));
    }
    block_frame(&mut y, &mut text);
    assert_eq!(text, "Hello\nworld");
}
