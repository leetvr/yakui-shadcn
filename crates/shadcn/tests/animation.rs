//! Exercise retained motion through real application state, input and paint.
use std::time::Duration;
use yakui::{Alignment, Rect, Vec2, WidgetId, Yakui, event::Event, input::MouseButton};
use yakui_shadcn::{
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

fn nodes(y: &Yakui, suffix: &str) -> Vec<WidgetId> {
    fn visit(y: &Yakui, id: WidgetId, suffix: &str, out: &mut Vec<WidgetId>) {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with(suffix) {
            out.push(id);
        }
        for &child in &node.children {
            visit(y, child, suffix, out);
        }
    }
    let mut out = Vec::new();
    visit(y, y.dom().root(), suffix, &mut out);
    out
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
#[test]
fn looping_carousel_wraps_one_slot_and_reverses_without_replacing_slides() {
    use yakui::input::KeyCode::*;
    let mut y = yak();
    let mut state = ui::CarouselState::default();
    state.looping = true;
    state.index = 2;
    let (start, _) = carousel_frame(&mut y, &mut state, 0);
    let track = nodes(&y, "carousel::Track")[0];
    let ids = y.dom().get(track).unwrap().children.clone();
    y.request_focus(Some(nodes(&y, "carousel::Swipe")[0]));
    key(&mut y, ArrowRight);
    carousel_frame(&mut y, &mut state, 1000);
    assert_eq!(state.index, 0);
    let (middle, _) = carousel_frame(&mut y, &mut state, 1075);
    assert!(middle[2] < start[2]);
    assert!(middle[0] > start[2] && middle[0] < start[2] + 320.);
    key(&mut y, ArrowLeft);
    let (reversed, _) = carousel_frame(&mut y, &mut state, 1075);
    assert_eq!(state.index, 2);
    assert_eq!(middle, reversed);
    let (end, active) = carousel_frame(&mut y, &mut state, 1375);
    assert!(!active);
    assert_eq!(end, start);
    assert_eq!(y.dom().get(track).unwrap().children, ids);
}
#[test]
fn looping_multislide_carousel_preserves_hit_targets_and_cyclic_keyboard_order() {
    fn frame(
        y: &mut Yakui,
        state: &mut ui::CarouselState,
        ms: u64,
        visible: usize,
        enabled: bool,
    ) -> Vec<ui::ControlResponse> {
        let mut buttons = Vec::new();
        y.start();
        animation::with_animation_time(time(ms), || {
            animation::with_motion_enabled(enabled, || {
                yakui::align(Alignment::TOP_LEFT, || {
                    ui::carousel(
                        state,
                        3,
                        ui::CarouselOptions {
                            visible,
                            ..Default::default()
                        },
                        |i| {
                            buttons.push(ui::button(format!("Slide {i}")));
                        },
                    );
                });
            });
        });
        y.finish();
        buttons
    }
    let mut y = yak();
    let mut state = ui::CarouselState::default();
    state.looping = true;
    state.index = 2;
    let buttons = frame(&mut y, &mut state, 0, 2, true);
    assert_eq!(buttons.len(), 3);
    let positions: Vec<_> = buttons
        .iter()
        .map(|b| y.layout_dom().get(b.id.unwrap()).unwrap().rect.pos().x)
        .collect();
    assert!((positions[0] - positions[2] - 160.).abs() < 0.01);
    click(&mut y, buttons[0].id.unwrap());
    assert!(frame(&mut y, &mut state, 0, 2, true)[0].clicked);
    y.request_focus(buttons[2].id);
    frame(&mut y, &mut state, 0, 2, true);
    key(&mut y, yakui::input::KeyCode::Tab);
    frame(&mut y, &mut state, 0, 2, true);
    assert!(frame(&mut y, &mut state, 0, 2, true)[0].focused);
    state.index = 0;
    frame(&mut y, &mut state, 1000, 2, true);
    let midway = frame(&mut y, &mut state, 1100, 2, true);
    click(&mut y, midway[0].id.unwrap());
    assert!(frame(&mut y, &mut state, 1100, 2, true)[0].clicked);
    frame(&mut y, &mut state, 1100, 2, false);
    assert!(!animation::animations_active(&y));
    // When every slide fits there is no offscreen slot for periodic relocation.
    // The entire catalogue remains visible and cyclic navigation stays at zero.
    state.index = 2;
    frame(&mut y, &mut state, 1200, 3, false);
    assert_eq!(state.index, 0);
}
fn toast_positions(y: &Yakui) -> Vec<f32> {
    let viewport = nodes(y, "toast::ToastViewport")[0];
    y.dom()
        .get(viewport)
        .unwrap()
        .children
        .iter()
        .filter(|&&id| !y.dom().get(id).unwrap().children.is_empty())
        .map(|&id| y.layout_dom().get(id).unwrap().rect.pos().y)
        .collect()
}
#[test]
fn toast_stack_reflows_from_current_positions_on_insert_remove_and_interruption() {
    let mut y = yak();
    let mut manager = ui::ToastManager::default();
    manager.add("First", "Saved", ui::ToastKind::Success);
    toast_frame(&mut y, &mut manager, 0);
    toast_frame(&mut y, &mut manager, 200);
    let start = toast_positions(&y)[0];
    let second = manager.add("Second", "Saved", ui::ToastKind::Success);
    toast_frame(&mut y, &mut manager, 200);
    assert_eq!(toast_positions(&y)[0], start);
    toast_frame(&mut y, &mut manager, 275);
    let middle = toast_positions(&y)[0];
    assert!(middle < start);
    manager.close(second);
    toast_frame(&mut y, &mut manager, 275);
    assert_eq!(toast_positions(&y)[0], middle);
    toast_frame(&mut y, &mut manager, 350);
    assert!(toast_positions(&y)[0] > middle);
    toast_frame(&mut y, &mut manager, 475);
    assert_eq!(toast_positions(&y), vec![start]);
    assert!(!animation::animations_active(&y));
    // Remove the older row after two are settled: the survivor must keep its
    // position even though its declaration now occupies the first child slot.
    manager.add("Third", "Saved", ui::ToastKind::Success);
    toast_frame(&mut y, &mut manager, 500);
    toast_frame(&mut y, &mut manager, 700);
    manager.close(manager.toasts[0].id);
    toast_frame(&mut y, &mut manager, 700);
    toast_frame(&mut y, &mut manager, 900);
    assert_eq!(toast_positions(&y), vec![start]);
    assert!(!animation::animations_active(&y));
}
#[test]
fn line_tabs_crossfade_underlines_after_real_keyboard_selection() {
    fn frame(y: &mut Yakui, selected: &mut usize, ms: u64, enabled: bool) -> Vec<f32> {
        y.start();
        animation::with_animation_time(time(ms), || {
            animation::with_motion_enabled(enabled, || {
                yakui::align(Alignment::TOP_LEFT, || {
                    ui::tabs(
                        &["First", "Second"],
                        selected,
                        ui::TabsVariant::Line,
                        |_| {},
                    );
                });
            });
        });
        y.finish();
        let rects: Vec<_> = nodes(y, "navigation::ChoiceWidget")
            .iter()
            .map(|&id| y.layout_dom().get(id).unwrap().rect)
            .collect();
        let paint = y.paint();
        rects
            .iter()
            .map(|rect| {
                paint
                    .layers()
                    .iter()
                    .flat_map(|l| &l.calls)
                    .flat_map(|c| &c.vertices)
                    .filter(|v| {
                        v.position.y * 400. > rect.max().y + 3.9
                            && v.position.y * 400. < rect.max().y + 4.1
                            && v.position.x * 600. >= rect.pos().x
                            && v.position.x * 600. <= rect.max().x
                    })
                    .map(|v| v.color.w)
                    .fold(0., f32::max)
            })
            .collect()
    }
    let mut y = yak();
    let mut selected = 0;
    assert_eq!(frame(&mut y, &mut selected, 0, true), vec![1., 0.]);
    y.request_focus(Some(nodes(&y, "navigation::ChoiceWidget")[0]));
    key(&mut y, yakui::input::KeyCode::ArrowRight);
    assert_eq!(frame(&mut y, &mut selected, 1000, true), vec![1., 0.]);
    assert_eq!(selected, 1);
    let middle = frame(&mut y, &mut selected, 1075, true);
    assert!(middle.iter().all(|&alpha| alpha > 0. && alpha < 1.));
    key(&mut y, yakui::input::KeyCode::ArrowLeft);
    assert_eq!(frame(&mut y, &mut selected, 1075, true), middle);
    assert_eq!(frame(&mut y, &mut selected, 1075, false), vec![1., 0.]);
}
#[test]
fn sidebar_keeps_label_nodes_and_control_focus_through_collapse_and_reversal() {
    fn frame(y: &mut Yakui, state: &mut ui::SidebarState, ms: u64) -> Vec<WidgetId> {
        y.start();
        animation::with_animation_time(time(ms), || {
            ui::sidebar(
                Vec2::new(600., 400.),
                state,
                "Menu",
                &[ui::SidebarLink::new("Settings", ui::Icon::Settings)],
                |_| {},
            );
        });
        y.finish();
        nodes(y, "sidebar::Label")
    }
    let mut y = yak();
    let mut state = ui::SidebarState::default();
    let labels = frame(&mut y, &mut state, 0);
    let toggle = nodes(&y, "interaction::ControlWidget")[0];
    click(&mut y, toggle);
    frame(&mut y, &mut state, 1000);
    assert!(state.collapsed);
    frame(&mut y, &mut state, 1000);
    let middle = frame(&mut y, &mut state, 1150);
    assert_eq!(middle, labels);
    let width = y.layout_dom().get(toggle).unwrap().rect.size().x;
    assert!(width > 32. && width < 224.);
    key(&mut y, yakui::input::KeyCode::Space);
    frame(&mut y, &mut state, 1150);
    frame(&mut y, &mut state, 1150);
    assert!(!state.collapsed);
    assert_eq!(y.layout_dom().get(toggle).unwrap().rect.size().x, width);
    assert_eq!(frame(&mut y, &mut state, 1350), labels);
    assert_eq!(y.layout_dom().get(toggle).unwrap().rect.size().x, 224.);
}

#[test]
fn toast_actions_follow_reflow_and_surviving_buttons_keep_their_identity() {
    fn frame(y: &mut Yakui, manager: &mut ui::ToastManager, ms: u64) -> ui::ToastResponse {
        let mut response = Default::default();
        y.start();
        animation::with_animation_time(time(ms), || {
            response = ui::toast_viewport(manager);
        });
        y.finish();
        response
    }
    let mut y = yak();
    let mut manager = ui::ToastManager::default();
    let first = manager.add("First", "Saved", ui::ToastKind::Success);
    let second = manager.add("Second", "Saved", ui::ToastKind::Success);
    for toast in &mut manager.toasts {
        toast.action = Some("Undo".into());
    }
    frame(&mut y, &mut manager, 0);
    frame(&mut y, &mut manager, 200);
    let buttons = nodes(&y, "interaction::ControlWidget");
    let second_action = buttons[2];
    y.request_focus(Some(second_action));
    manager.close(first);
    frame(&mut y, &mut manager, 200);
    frame(&mut y, &mut manager, 400);
    assert_eq!(nodes(&y, "interaction::ControlWidget")[0], second_action);
    key(&mut y, yakui::input::KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut manager, 400).action, Some(second));
    let third = manager.add("Third", "Saved", ui::ToastKind::Success);
    frame(&mut y, &mut manager, 400);
    frame(&mut y, &mut manager, 475);
    click(&mut y, second_action);
    assert_eq!(frame(&mut y, &mut manager, 475).action, Some(second));
    // The newest toast reused the first slot, but Tab still follows display order.
    for _ in 0..2 {
        key(&mut y, yakui::input::KeyCode::Tab);
        frame(&mut y, &mut manager, 600);
    }
    key(&mut y, yakui::input::KeyCode::Enter);
    assert_eq!(frame(&mut y, &mut manager, 600).dismissed, Some(third));
}
#[test]
fn shorthand_tooltip_retains_exit_paint_after_pointer_leaves() {
    fn frame(y: &mut Yakui, ms: u64) -> ui::ControlResponse {
        let mut trigger = Default::default();
        y.start();
        animation::with_animation_time(time(ms), || {
            yakui::center(|| {
                trigger = ui::button("Hover");
            });
            ui::tooltip(trigger, "Helpful text");
        });
        y.finish();
        trigger
    }
    let mut y = yak();
    let trigger = frame(&mut y, 0);
    let rect = y.layout_dom().get(trigger.id.unwrap()).unwrap().rect;
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    frame(&mut y, 1000);
    frame(&mut y, 1100);
    assert!(!nodes(&y, "tooltip::Arrow").is_empty());
    y.handle_event(Event::CursorMoved(None));
    frame(&mut y, 1100);
    frame(&mut y, 1150);
    assert!(!nodes(&y, "tooltip::Arrow").is_empty());
    assert!(
        y.paint()
            .layers()
            .iter()
            .flat_map(|l| &l.calls)
            .flat_map(|c| &c.vertices)
            .any(|v| v.opacity > 0. && v.opacity < 1.)
    );
    frame(&mut y, 1200);
    assert!(nodes(&y, "tooltip::Arrow").is_empty());
}
