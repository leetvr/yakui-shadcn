//! Real root overlays and keyboard focus cycles.
use yakui::{
    Vec2, Yakui,
    event::Event,
    input::{KeyCode, MouseButton},
};
use yakui_adequate_ui as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    y.set_surface_size(Vec2::new(800., 600.));
    y.set_unscaled_viewport(yakui::Rect::from_pos_size(
        Vec2::ZERO,
        Vec2::new(800., 600.),
    ));
    ui::add_fonts(&mut y);
    y
}
fn frame(y: &mut Yakui, body: &mut impl FnMut()) {
    y.start();
    body();
    y.finish();
}
fn key(y: &mut Yakui, key: KeyCode) {
    y.handle_event(Event::KeyChanged {
        key,
        down: true,
        modifiers: None,
    });
    y.handle_event(Event::KeyChanged {
        key,
        down: false,
        modifiers: None,
    });
}
#[test]
fn dialog_autofocus_escape_and_restore_trigger() {
    let mut y = yak();
    let mut state = ui::OverlayState::default();
    let mut inside = ui::ControlResponse::default();
    let mut trigger = ui::ControlResponse::default();
    let mut build = || {
        ui::with_theme(ui::Theme::light(), || {
            yakui::align(yakui::Alignment::TOP_LEFT, || {
                trigger = ui::button("Open");
                if trigger.clicked {
                    state.open_from(trigger);
                }
            });
            ui::dialog(&mut state, |_| {
                inside = ui::button("Save");
            });
        })
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Enter);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Escape);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    drop(build);
    assert!(!state.open);
    assert!(trigger.focused);
    assert!(inside.focused);
}
#[test]
fn dialog_backdrop_dismisses_but_content_does_not() {
    let mut y = yak();
    let mut state = ui::OverlayState::default();
    state.open = true;
    let mut build = || {
        ui::dialog(&mut state, |_| {
            ui::label("Dialog");
            ui::button("Save");
        })
    };
    frame(&mut y, &mut build);
    y.handle_event(Event::CursorMoved(Some(Vec2::new(400., 300.))));
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: true,
    });
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: false,
    });
    frame(&mut y, &mut build);
    y.handle_event(Event::CursorMoved(Some(Vec2::new(10., 10.))));
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: true,
    });
    y.handle_event(Event::MouseButtonChanged {
        button: MouseButton::One,
        down: false,
    });
    frame(&mut y, &mut build);
    drop(build);
    assert!(!state.open);
}
#[test]
fn arrow_keys_select_tabs_and_wrap() {
    let mut y = yak();
    let mut selected = 0;
    let mut build = || {
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            ui::tabs(
                &["Account", "Password"],
                &mut selected,
                ui::TabsVariant::Default,
                |_| {},
            )
        });
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::ArrowRight);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(selected, 1);
}

#[test]
fn modal_tab_wraps_inside_and_editor_escape_dismisses() {
    let mut y = yak();
    let mut state = ui::OverlayState::default();
    state.open = true;
    let mut first = ui::ControlResponse::default();
    let mut last = ui::ControlResponse::default();
    let mut build = || {
        ui::dialog(&mut state, |_| {
            first = ui::button("First");
            last = ui::button("Last");
        })
    };
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    drop(build);
    assert!(first.focused);
    assert!(!last.focused);
    let mut text = String::new();
    let mut build = || {
        ui::dialog(&mut state, |_| {
            ui::input(&mut text, "Name");
        })
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Escape);
    frame(&mut y, &mut build);
    drop(build);
    assert!(!state.open);
}

#[test]
fn modal_pointer_controls_activate_before_the_backdrop_and_outside_dismisses() {
    fn build(y: &mut Yakui, state: &mut ui::OverlayState, saved: &mut u32) -> ui::ControlResponse {
        y.start();
        let mut save = ui::ControlResponse::default();
        ui::dialog(state, |_| {
            save = ui::button("Save");
            if save.clicked {
                *saved += 1;
            }
            ui::text("Editable content");
        });
        y.finish();
        save
    }
    let mut y = yak();
    let mut state = ui::OverlayState::default();
    state.open = true;
    let mut saved = 0;
    let save = build(&mut y, &mut state, &mut saved);
    let rect = y.layout_dom().get(save.id.unwrap()).unwrap().rect;
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
    build(&mut y, &mut state, &mut saved);
    build(&mut y, &mut state, &mut saved);
    assert_eq!(saved, 1);
    assert!(state.open);
    y.handle_event(Event::CursorMoved(Some(Vec2::new(10., 10.))));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
    build(&mut y, &mut state, &mut saved);
    assert!(!state.open);
    assert_eq!(saved, 1);
}
