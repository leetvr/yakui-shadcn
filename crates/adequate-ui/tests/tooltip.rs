//! A pointer-only tooltip must not acquire keyboard focus as it closes.
use yakui::{Rect, Vec2, Yakui, event::Event};
use yakui_adequate_ui as ui;
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
