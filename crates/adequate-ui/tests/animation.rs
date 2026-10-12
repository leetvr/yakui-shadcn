//! Exercise retained motion through real application state, input and paint.
use std::time::Duration;
use yakui::{Alignment, Rect, Vec2, WidgetId, Yakui, event::Event, input::MouseButton};
use yakui_adequate_ui::{
    self as ui,
    animation::{self, Easing, Motion, Transition},
};
fn time(ms: u64) -> Duration {
    Duration::from_millis(ms)
}
fn yak() -> Yakui {
    let mut y = Yakui::new();
    y.set_surface_size(Vec2::new(600., 400.));
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, Vec2::new(600., 400.)));
    ui::add_fonts(&mut y);
    y
}
fn click(y: &mut Yakui, id: WidgetId) {
    let position = y.layout_dom().get(id).unwrap().rect.center();
    y.handle_event(Event::CursorMoved(Some(position)));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
}
fn switch_frame(
    y: &mut Yakui,
    value: &mut bool,
    ms: u64,
    enabled: bool,
) -> (ui::ControlResponse, bool) {
    let mut response = ui::ControlResponse::default();
    y.start();
    animation::with_animation_time(time(ms), || {
        animation::with_motion_enabled(enabled, || {
            yakui::align(Alignment::TOP_LEFT, || {
                response = ui::Switch::new().show(value)
            });
        })
    });
    y.finish();
    (response, animation::animations_active(y))
}
fn thumb_x(y: &Yakui) -> f32 {
    fn find(y: &Yakui, id: WidgetId) -> Option<f32> {
        let node = y.dom().get(id).unwrap();
        let rect = y.layout_dom().get(id).unwrap().rect;
        if node.widget.type_name().contains("RoundRect") && rect.size() == Vec2::splat(16.) {
            return Some(rect.pos().x);
        }
        node.children.iter().find_map(|&id| find(y, id))
    }
    find(y, y.dom().root()).expect("actual switch thumb")
}
#[test]
fn transition_retargets_without_jumping_and_samples_independently_of_frame_rate() {
    let motion = Motion::new(time(200), Easing::Linear);
    let mut transition = Transition::new(0.);
    transition.set_target(100., motion, time(1000));
    assert_eq!(transition.value(time(1050)), 25.);
    let before = transition.value(time(1075));
    transition.set_target(0., motion, time(1075));
    assert_eq!(transition.value(time(1075)), before);
    assert_eq!(transition.value(time(1175)), before * 0.5);
    let mut sparse = transition;
    for ms in 1075..1275 {
        let _ = transition.value(time(ms));
    }
    assert_eq!(transition.value(time(1275)), sparse.value(time(1275)));
    sparse.snap(42.);
    assert_eq!(sparse.value(time(1275)), 42.);
    assert!(!sparse.is_active(time(1275)));
}
#[test]
fn css_easing_solves_time_axis_and_preserves_endpoints() {
    assert_eq!(Easing::Standard.sample(0.), 0.);
    assert_eq!(Easing::Standard.sample(1.), 1.);
    assert!((Easing::Standard.sample(0.5) - 0.77556).abs() < 0.0001);
    assert!((Easing::EaseInOut.sample(0.5) - 0.5).abs() < 0.0001);
}
#[test]
fn native_switch_updates_value_immediately_and_reverses_from_current_thumb_position() {
    let mut y = yak();
    let mut value = false;
    let (trigger, _) = switch_frame(&mut y, &mut value, 0, true);
    assert_eq!(thumb_x(&y), 1.);
    click(&mut y, trigger.id.unwrap());
    assert!(switch_frame(&mut y, &mut value, 1000, true).1);
    assert!(value);
    assert_eq!(thumb_x(&y), 1.);
    switch_frame(&mut y, &mut value, 1075, true);
    let middle = thumb_x(&y);
    assert!(middle > 1. && middle < 15.);
    click(&mut y, trigger.id.unwrap());
    switch_frame(&mut y, &mut value, 1075, true);
    assert!(!value);
    assert_eq!(thumb_x(&y), middle);
    assert!(!switch_frame(&mut y, &mut value, 1250, true).1);
    assert_eq!(thumb_x(&y), 1.);
    click(&mut y, trigger.id.unwrap());
    assert!(!switch_frame(&mut y, &mut value, 1300, false).1);
    assert!(value);
    assert_eq!(thumb_x(&y), 15.);
}
fn dialog_frame(
    y: &mut Yakui,
    state: &mut ui::OverlayState,
    actions: &mut usize,
    ms: u64,
) -> (ui::ControlResponse, ui::ControlResponse) {
    let (mut trigger, mut save) = Default::default();
    y.start();
    animation::with_animation_time(time(ms), || {
        yakui::align(Alignment::TOP_LEFT, || {
            trigger = ui::button("Open");
            if trigger.clicked {
                state.open_from(trigger);
            }
        });
        ui::dialog(state, |_| {
            ui::label("Dialog");
            save = ui::button("Save");
            if save.clicked {
                *actions += 1;
            }
        });
    });
    y.finish();
    (trigger, save)
}
#[test]
fn closing_dialog_retains_paint_returns_focus_and_disables_exit_content() {
    let mut y = yak();
    let mut state = ui::OverlayState::default();
    let mut actions = 0;
    let (trigger, _) = dialog_frame(&mut y, &mut state, &mut actions, 0);
    click(&mut y, trigger.id.unwrap());
    dialog_frame(&mut y, &mut state, &mut actions, 1000);
    let (_, save) = dialog_frame(&mut y, &mut state, &mut actions, 1100);
    assert!(save.focused);
    state.close();
    dialog_frame(&mut y, &mut state, &mut actions, 1100);
    let (restored, save) = dialog_frame(&mut y, &mut state, &mut actions, 1150);
    assert!(restored.focused);
    assert!(!save.focused);
    let paint = y.paint();
    assert!(
        paint
            .layers()
            .iter()
            .flat_map(|layer| &layer.calls)
            .flat_map(|call| &call.vertices)
            .any(|v| v.opacity > 0. && v.opacity < 1.)
    );
    click(&mut y, save.id.unwrap());
    dialog_frame(&mut y, &mut state, &mut actions, 1175);
    assert_eq!(actions, 0);
    state.open_from(trigger);
    dialog_frame(&mut y, &mut state, &mut actions, 1175);
    let (_, save) = dialog_frame(&mut y, &mut state, &mut actions, 1300);
    assert!(save.focused);
    click(&mut y, save.id.unwrap());
    dialog_frame(&mut y, &mut state, &mut actions, 1310);
    assert_eq!(actions, 1);
}

fn accordion_frame(y: &mut Yakui, open: &mut bool, ms: u64) -> (f32, bool) {
    y.start();
    animation::with_animation_time(time(ms), || {
        yakui::align(Alignment::TOP_LEFT, || {
            ui::accordion_item("Details", open, 300., || ui::label("Retained content"))
        });
    });
    y.finish();
    fn height(y: &Yakui, id: WidgetId) -> Option<f32> {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("animation::Reveal") {
            return Some(y.layout_dom().get(id).unwrap().rect.size().y);
        }
        node.children.iter().find_map(|&id| height(y, id))
    }
    (
        height(y, y.dom().root()).unwrap(),
        animation::animations_active(y),
    )
}
#[test]
fn accordion_animates_measured_height_and_reports_layout_motion_after_finish() {
    let mut y = yak();
    let mut open = false;
    assert_eq!(accordion_frame(&mut y, &mut open, 0).0, 0.);
    open = true;
    let (start, active) = accordion_frame(&mut y, &mut open, 1000);
    assert_eq!(start, 0.);
    assert!(active);
    let middle = accordion_frame(&mut y, &mut open, 1100).0;
    let (end, active) = accordion_frame(&mut y, &mut open, 1200);
    assert!(!active);
    assert!(middle > 0. && middle < end);
    open = false;
    assert_eq!(accordion_frame(&mut y, &mut open, 1200).0, end);
    assert!(accordion_frame(&mut y, &mut open, 1300).0 < end);
    assert_eq!(accordion_frame(&mut y, &mut open, 1400).0, 0.);
}

fn carousel_frame(y: &mut Yakui, state: &mut ui::CarouselState, ms: u64) -> (Vec<f32>, bool) {
    y.start();
    animation::with_animation_time(time(ms), || {
        yakui::align(Alignment::TOP_LEFT, || {
            ui::carousel(state, 3, Default::default(), |index| {
                ui::label(format!("Slide {index}"));
            })
        });
    });
    y.finish();
    fn positions(y: &Yakui, id: WidgetId) -> Option<Vec<f32>> {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("carousel::Track") {
            return Some(
                node.children
                    .iter()
                    .map(|&child| y.layout_dom().get(child).unwrap().rect.pos().x)
                    .collect(),
            );
        }
        node.children.iter().find_map(|&child| positions(y, child))
    }
    (
        positions(y, y.dom().root()).unwrap(),
        animation::animations_active(y),
    )
}
#[test]
fn carousel_moves_retained_slide_layout_without_changing_application_selection() {
    let mut y = yak();
    let mut state = ui::CarouselState::default();
    let (start, _) = carousel_frame(&mut y, &mut state, 0);
    assert_eq!(start.len(), 3);
    state.index = 1;
    assert!(carousel_frame(&mut y, &mut state, 1000).1);
    let (middle, _) = carousel_frame(&mut y, &mut state, 1100);
    assert_eq!(state.index, 1);
    assert!(middle[0] < start[0]);
    let (end, active) = carousel_frame(&mut y, &mut state, 1400);
    assert!(!active);
    assert!(middle[0] > end[0]);
    assert!((end[1] - start[0]).abs() < 0.01);
}
fn toast_frame(y: &mut Yakui, manager: &mut ui::ToastManager, ms: u64) -> bool {
    y.start();
    animation::with_animation_time(time(ms), || {
        ui::toast_viewport(manager);
    });
    y.finish();
    animation::animations_active(y)
}
#[test]
fn toast_removal_updates_business_state_immediately_and_keeps_exit_paint() {
    let mut y = yak();
    let mut manager = ui::ToastManager::default();
    let id = manager.add("Saved", "Your changes were saved.", ui::ToastKind::Success);
    assert!(toast_frame(&mut y, &mut manager, 0));
    assert!(!toast_frame(&mut y, &mut manager, 200));
    manager.close(id);
    assert!(manager.toasts.is_empty());
    assert!(toast_frame(&mut y, &mut manager, 200));
    assert!(toast_frame(&mut y, &mut manager, 300));
    assert!(
        y.paint()
            .layers()
            .iter()
            .flat_map(|layer| &layer.calls)
            .flat_map(|call| &call.vertices)
            .any(|v| v.opacity > 0. && v.opacity < 1.)
    );
    assert!(!toast_frame(&mut y, &mut manager, 400));
    assert!(
        y.paint()
            .layers()
            .iter()
            .all(|layer| layer.calls.is_empty())
    );
}
#[test]
fn sheet_hit_targets_move_with_the_panel_during_entry() {
    fn frame(y: &mut Yakui, state: &mut ui::OverlayState, ms: u64) -> Option<WidgetId> {
        let mut button = None;
        y.start();
        animation::with_animation_time(time(ms), || {
            ui::sheet(state, ui::Side::Right, |_| {
                button = ui::button("Save").id;
            })
        });
        y.finish();
        button
    }
    let mut y = yak();
    let mut state = ui::OverlayState::default();
    frame(&mut y, &mut state, 0);
    state.open = true;
    let id = frame(&mut y, &mut state, 1000).unwrap();
    let start = y.layout_dom().get(id).unwrap().rect.pos().x;
    frame(&mut y, &mut state, 1100);
    let middle = y.layout_dom().get(id).unwrap().rect.pos().x;
    frame(&mut y, &mut state, 1200);
    let end = y.layout_dom().get(id).unwrap().rect.pos().x;
    assert!(start > middle && middle > end);
    assert!((start - end - 384.).abs() < 0.01);
}
