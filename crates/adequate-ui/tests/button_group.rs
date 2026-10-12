//! Common group geometry and real control interaction, without simulated widgets.
use yakui::{Alignment, Rect, Vec2, Yakui, event::Event, input::MouseButton};
use yakui_adequate_ui as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    let size = Vec2::new(600., 400.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    y
}
fn frame(y: &mut Yakui, orientation: ui::Orientation) -> [ui::ControlResponse; 2] {
    y.start();
    let mut responses = [ui::ControlResponse::default(); 2];
    yakui::align(Alignment::TOP_LEFT, || {
        ui::ButtonGroup::new().orientation(orientation).show(|| {
            responses[0] = ui::Button::new("")
                .variant(ui::Variant::Outline)
                .size(ui::ButtonSize::Icon)
                .icon(ui::Icon::Plus)
                .show();
            responses[1] = ui::Button::new("")
                .variant(ui::Variant::Outline)
                .size(ui::ButtonSize::Icon)
                .icon(ui::Icon::Minus)
                .show();
        })
    });
    y.finish();
    responses
}
#[test]
fn joined_buttons_keep_their_own_targets_and_share_one_border_on_each_axis() {
    for orientation in [ui::Orientation::Horizontal, ui::Orientation::Vertical] {
        let mut y = yak();
        let controls = frame(&mut y, orientation);
        let first = y.layout_dom().get(controls[0].id.unwrap()).unwrap().rect;
        let second = y.layout_dom().get(controls[1].id.unwrap()).unwrap().rect;
        assert_eq!(first.size(), Vec2::splat(32.));
        assert_eq!(second.size(), Vec2::splat(32.));
        assert_eq!(
            second.pos(),
            if orientation == ui::Orientation::Horizontal {
                Vec2::new(31., 0.)
            } else {
                Vec2::new(0., 31.)
            }
        );
        y.handle_event(Event::CursorMoved(Some(second.center())));
        for down in [true, false] {
            y.handle_event(Event::MouseButtonChanged {
                button: MouseButton::One,
                down,
            });
        }
        let clicked = frame(&mut y, orientation);
        assert!(!clicked[0].clicked);
        assert!(clicked[1].clicked);
        assert!(!frame(&mut y, orientation)[1].clicked);
    }
}
#[test]
fn nested_groups_keep_an_eight_pixel_gap_and_editable_inputs_keep_caller_state() {
    fn frame(y: &mut Yakui, text: &mut String) -> [ui::ControlResponse; 2] {
        y.start();
        let mut controls = [ui::ControlResponse::default(); 2];
        yakui::align(Alignment::TOP_LEFT, || {
            ui::button_group(|| {
                ui::button_group(|| {
                    controls[0] = ui::button("Back");
                });
                ui::button_group(|| {
                    ui::width(180., || {
                        ui::input(text, "Search...");
                    });
                    controls[1] = ui::button("Search");
                });
            })
        });
        y.finish();
        controls
    }
    let mut y = yak();
    let mut text = String::new();
    let controls = frame(&mut y, &mut text);
    let first = y.layout_dom().get(controls[0].id.unwrap()).unwrap().rect;
    fn editor(y: &Yakui, id: yakui::WidgetId) -> Option<yakui::WidgetId> {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("::Editor") {
            return Some(id);
        }
        node.children.iter().find_map(|&child| editor(y, child))
    }
    let input = editor(&y, y.dom().root()).unwrap();
    let rect = y.layout_dom().get(input).unwrap().rect;
    assert!((rect.pos().x - first.max().x - 8.).abs() < 0.01);
    y.request_focus(Some(input));
    for letter in "query".chars() {
        y.handle_event(Event::TextInput(letter));
    }
    frame(&mut y, &mut text);
    assert_eq!(text, "query");
}
