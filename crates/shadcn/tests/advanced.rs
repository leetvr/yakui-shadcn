//! Native events, layout and caller state across the more complex controls.
use yakui::{Vec2, Yakui, event::Event, input::KeyCode};
use yakui_shadcn as ui;
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
    yakui::align(yakui::Alignment::TOP_LEFT, body);
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
fn find(y: &Yakui, name: &str) -> yakui::WidgetId {
    fn visit(y: &Yakui, id: yakui::WidgetId, name: &str) -> Option<yakui::WidgetId> {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with(name) {
            return Some(id);
        }
        node.children
            .iter()
            .find_map(|child| visit(y, *child, name))
    }
    visit(y, y.dom().root(), name).unwrap()
}
#[test]
fn slider_keyboard_respects_step_and_ordered_neighbours() {
    let mut y = yak();
    let mut values = [20., 30.];
    let mut build = || {
        ui::range_slider(
            &mut values,
            ui::SliderOptions {
                step: 5.,
                ..Default::default()
            },
        );
    };
    frame(&mut y, &mut build);
    y.request_focus(Some(find(&y, "::SliderWidget")));
    key(&mut y, KeyCode::ArrowRight);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::End);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(values, [30., 30.]);
}
#[test]
fn otp_filters_limits_and_backspaces_native_text() {
    let mut y = yak();
    let mut value = String::new();
    let mut responses = Vec::new();
    let mut build = || {
        responses.push(ui::input_otp(&mut value, ui::OtpOptions::default()));
    };
    frame(&mut y, &mut build);
    y.request_focus(Some(find(&y, "::OtpWidget")));
    for c in "12a34567".chars() {
        y.handle_event(Event::TextInput(c));
    }
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Backspace);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(value, "12345");
    assert!(responses.iter().any(|r| r.complete));
}
#[test]
fn resizable_keyboard_changes_application_ratio_with_bounds() {
    let mut y = yak();
    let mut ratio = 0.5;
    let mut build = || {
        ui::resizable(
            Vec2::new(400., 200.),
            ui::Orientation::Horizontal,
            &mut ratio,
            || ui::label("One"),
            || ui::label("Two"),
        )
    };
    frame(&mut y, &mut build);
    y.request_focus(Some(find(&y, "::Handle")));
    key(&mut y, KeyCode::End);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(ratio, 0.9);
}
#[test]
fn carousel_keyboard_advances_and_stops_at_the_last_slide() {
    let mut y = yak();
    let mut state = ui::CarouselState::default();
    let mut build = || {
        ui::carousel(&mut state, 3, ui::CarouselOptions::default(), |i| {
            ui::label(i.to_string())
        })
    };
    frame(&mut y, &mut build);
    y.request_focus(Some(find(&y, "::Swipe")));
    for _ in 0..4 {
        key(&mut y, KeyCode::ArrowRight);
        frame(&mut y, &mut build);
    }
    drop(build);
    assert_eq!(state.index, 2);
}
#[test]
fn charts_report_keyboard_inspection_and_activation() {
    let mut y = yak();
    let mut response = ui::ChartResponse::default();
    let mut build = || {
        response = ui::Chart::new(
            vec![
                ui::ChartPoint::new("Jan", vec![1.]),
                ui::ChartPoint::new("Feb", vec![2.]),
            ],
            vec![ui::ChartSeries::new("Visitors", ui::Color::BLACK)],
        )
        .show();
    };
    frame(&mut y, &mut build);
    y.request_focus(Some(find(&y, "::PlotWidget")));
    key(&mut y, KeyCode::ArrowRight);
    key(&mut y, KeyCode::Enter);
    frame(&mut y, &mut build);
    drop(build);
    assert_eq!(response.hovered, Some(1));
    assert_eq!(response.clicked, Some(1));
}
#[test]
fn transcript_prepend_preserves_visible_ids_and_scroll_anchor() {
    let mut y = yak();
    let state = std::cell::RefCell::new(ui::MessageScrollerState::default());
    let messages = std::cell::RefCell::new(vec![2u64, 3, 4, 5, 6]);
    let mut build = || {
        ui::message_scroller(Vec2::new(300., 150.), &mut state.borrow_mut(), || {
            for id in messages.borrow().iter() {
                ui::message_scroller_item(*id, || {
                    ui::size(Vec2::new(300., 60.), || ui::label(format!("Message {id}")))
                });
            }
        });
    };
    frame(&mut y, &mut build);
    state.borrow_mut().scroll_to_message(3);
    frame(&mut y, &mut build);
    let visible = state.borrow().visible_message_ids();
    let old = state.borrow().position();
    messages.borrow_mut().insert(0, 1);
    frame(&mut y, &mut build);
    assert_eq!(state.borrow().visible_message_ids(), visible);
    assert!(state.borrow().position() > old);
}
#[test]
fn required_questionnaire_answers_are_not_valid_until_supplied() {
    let q = ui::Question {
        name: "name".into(),
        title: "Name?".into(),
        description: String::new(),
        choices: vec![],
        required: true,
        multiple: false,
        allow_other: true,
    };
    let mut state = ui::QuestionnaireState::default();
    assert!(!state.valid(0, &q));
    state.answers.push(ui::Answer {
        other: "Kane".into(),
        ..Default::default()
    });
    assert!(state.valid(0, &q));
}

#[test]
fn range_slider_tab_reaches_the_second_thumb_before_leaving() {
    let mut y = yak();
    let values = std::cell::RefCell::new([20., 80.]);
    let after_focused = std::cell::Cell::new(false);
    let mut build = || {
        ui::column(12., || {
            ui::range_slider(&mut *values.borrow_mut(), ui::SliderOptions::default());
            after_focused.set(ui::button("After").focused);
        })
    };
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Tab);
    key(&mut y, KeyCode::ArrowRight);
    frame(&mut y, &mut build);
    assert_eq!(*values.borrow(), [20., 81.]);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    assert!(after_focused.get());
}

#[test]
fn progress_fill_is_sized_by_value_under_tight_track_constraints() {
    let mut y = yak();
    let mut build = || ui::progress(0.33, 300.);
    frame(&mut y, &mut build);
    fn rects(y: &Yakui, id: yakui::WidgetId, out: &mut Vec<yakui::Rect>) {
        let node = y.dom().get(id).unwrap();
        if node.widget.type_name().ends_with("::RoundRectWidget") {
            out.push(y.layout_dom().get(id).unwrap().rect);
        }
        for &child in &node.children {
            rects(y, child, out);
        }
    }
    let mut boxes = Vec::new();
    rects(&y, y.dom().root(), &mut boxes);
    assert_eq!(boxes[0].size(), Vec2::new(300., 4.));
    assert!((boxes[1].size().x - 99.).abs() < 0.001);
    assert_eq!(boxes[1].size().y, 4.);
}

#[test]
fn password_edits_graphemes_and_replaces_selection_in_caller_state() {
    let mut y = yak();
    let value = std::cell::RefCell::new(String::new());
    let id = std::cell::Cell::new(None);
    let mut build = || {
        id.set(
            ui::Input::new(&mut value.borrow_mut())
                .password(true)
                .show()
                .id,
        );
    };
    frame(&mut y, &mut build);
    y.request_focus(id.get());
    for c in "secret🦀e\u{301}".chars() {
        y.handle_event(Event::TextInput(c));
    }
    frame(&mut y, &mut build);
    key(&mut y, KeyCode::Backspace);
    frame(&mut y, &mut build);
    assert_eq!(&*value.borrow(), "secret🦀");
    y.handle_event(Event::KeyChanged {
        key: KeyCode::ArrowLeft,
        down: true,
        modifiers: Some(yakui::input::Modifiers::SHIFT),
    });
    y.handle_event(Event::KeyChanged {
        key: KeyCode::ArrowLeft,
        down: false,
        modifiers: Some(yakui::input::Modifiers::SHIFT),
    });
    y.handle_event(Event::ModifiersChanged(yakui::input::Modifiers::empty()));
    y.handle_event(Event::TextInput('!'));
    frame(&mut y, &mut build);
    assert_eq!(&*value.borrow(), "secret!");
}

#[test]
fn slider_drag_updates_caller_and_painted_thumb_in_the_same_frame_at_hidpi() {
    use std::cell::{Cell, RefCell};
    use yakui::input::MouseButton;
    for orientation in [ui::Orientation::Horizontal, ui::Orientation::Vertical] {
        let mut y = yak();
        y.set_scale_factor(2.);
        let value = RefCell::new(25.);
        let disabled = Cell::new(false);
        let response = Cell::new(ui::SliderResponse::default());
        let mut build = || {
            ui::animation::with_motion_enabled(false, || {
                response.set(ui::slider(
                    &mut value.borrow_mut(),
                    ui::SliderOptions {
                        length: 200.,
                        orientation,
                        disabled: disabled.get(),
                        ..Default::default()
                    },
                ));
            });
        };
        frame(&mut y, &mut build);
        let rect = y.layout_dom().get(find(&y, "::SliderWidget")).unwrap().rect;
        let point = |ratio: f32| {
            if orientation == ui::Orientation::Horizontal {
                Vec2::new(rect.pos().x + 6. + ratio * 188., rect.center().y)
            } else {
                Vec2::new(rect.center().x, rect.max().y - 6. - ratio * 188.)
            }
        };
        y.handle_event(Event::CursorMoved(Some(point(0.25) * 2.)));
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down: true,
        });
        y.handle_event(Event::CursorMoved(Some(point(0.75) * 2.)));
        frame(&mut y, &mut build);
        assert_eq!(*value.borrow(), 75.);
        assert!(response.get().changed);
        assert!(!response.get().confirmed);
        // Inspect the actual painted white thumb, not just the returned value.
        // A stale props copy used to paint the previous position for one frame.
        let surface = y.surface_size();
        let paint = y.paint();
        let white: Vec<_> = paint
            .layers()
            .iter()
            .flat_map(|layer| &layer.calls)
            .flat_map(|call| &call.vertices)
            .filter(|v| v.color == yakui::Color::WHITE.to_linear())
            .map(|v| v.position * surface / 2.)
            .collect();
        assert!(!white.is_empty());
        assert!(white.iter().all(|p| p.distance(point(0.75)) <= 6.01));
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down: false,
        });
        frame(&mut y, &mut build);
        assert!(response.get().confirmed);
        frame(&mut y, &mut build);
        assert!(!response.get().confirmed);
        disabled.set(true);
        frame(&mut y, &mut build);
        y.handle_event(Event::CursorMoved(Some(point(0.1) * 2.)));
        for down in [true, false] {
            y.handle_event(Event::MouseButtonChanged {
                button: MouseButton::One,
                down,
            });
        }
        key(&mut y, KeyCode::ArrowLeft);
        frame(&mut y, &mut build);
        assert_eq!(*value.borrow(), 75.);
        assert!(!response.get().changed);
    }
}

#[test]
fn slider_ring_follows_the_hovered_thumb_and_keyboard_focus() {
    let mut y = yak();
    let mut value = 75.;
    let mut build = || {
        ui::animation::with_motion_enabled(false, || {
            ui::slider(
                &mut value,
                ui::SliderOptions {
                    length: 200.,
                    ..Default::default()
                },
            );
        });
    };
    frame(&mut y, &mut build);
    let rect = y.layout_dom().get(find(&y, "::SliderWidget")).unwrap().rect;
    let has_ring = |y: &mut Yakui| {
        let surface = y.surface_size();
        y.paint()
            .layers()
            .iter()
            .flat_map(|layer| &layer.calls)
            .flat_map(|call| &call.vertices)
            .any(|v| v.position.y * surface.y > rect.center().y + 8.)
    };
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    frame(&mut y, &mut build);
    assert!(
        !has_ring(&mut y),
        "hovering the bare track must not highlight the thumb"
    );
    y.handle_event(Event::CursorMoved(Some(Vec2::new(
        rect.pos().x + 147.,
        rect.center().y,
    ))));
    frame(&mut y, &mut build);
    assert!(has_ring(&mut y));
    y.handle_event(Event::CursorMoved(None));
    frame(&mut y, &mut build);
    assert!(!has_ring(&mut y));
    key(&mut y, KeyCode::Tab);
    // Yakui applies deferred navigation at finish; the next update receives focus.
    frame(&mut y, &mut build);
    frame(&mut y, &mut build);
    assert!(has_ring(&mut y));
}
