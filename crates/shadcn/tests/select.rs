//! Actual native select geometry, keyboard focus and caller-owned values.
use yakui::{
    Rect, Vec2, Yakui,
    event::Event,
    input::{KeyCode, MouseButton},
    widgets::Pad,
};
use yakui_shadcn as ui;
fn yak() -> Yakui {
    let mut y = Yakui::new();
    let size = Vec2::new(600., 500.);
    y.set_surface_size(size);
    y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
    ui::add_fonts(&mut y);
    y
}
fn items() -> Vec<ui::SelectItem> {
    vec![
        ui::SelectItem::label("Fruit"),
        ui::SelectItem::empty("Choose..."),
        ui::SelectItem::new(10, "Apple"),
        ui::SelectItem::new(20, "Banana"),
        ui::SelectItem::new(25, "Grapes").disabled(true),
        ui::SelectItem::separator(),
        ui::SelectItem::label("Vegetables"),
        ui::SelectItem::new(30, "Carrot"),
    ]
}
fn frame(
    y: &mut Yakui,
    state: &mut ui::SelectState,
    disabled: bool,
    small: bool,
) -> (ui::ControlResponse, bool) {
    y.start();
    let items = items();
    let mut trigger = ui::ControlResponse::default();
    yakui::align(yakui::Alignment::TOP_LEFT, || {
        yakui::pad(
            Pad {
                left: 100.,
                top: 150.,
                right: 0.,
                bottom: 0.,
            },
            || {
                trigger = ui::Select::new(&items)
                    .width(192.)
                    .disabled(disabled)
                    .size(if small {
                        ui::SelectSize::Sm
                    } else {
                        ui::SelectSize::Default
                    })
                    .show(state);
            },
        );
    });
    let changed = ui::SelectContent::new(&items).show(state);
    y.finish();
    (trigger, changed)
}
fn key(y: &mut Yakui, key: KeyCode) {
    for down in [true, false] {
        y.handle_event(Event::KeyChanged {
            key,
            down,
            modifiers: None,
        });
    }
}
fn click(y: &mut Yakui, id: yakui::WidgetId) {
    let rect = y.layout_dom().get(id).unwrap().rect;
    y.handle_event(Event::CursorMoved(Some(rect.center())));
    for down in [true, false] {
        y.handle_event(Event::MouseButtonChanged {
            button: MouseButton::One,
            down,
        });
    }
}
fn entries(y: &Yakui) -> Vec<yakui::WidgetId> {
    fn visit(y: &Yakui, id: yakui::WidgetId, ids: &mut Vec<yakui::WidgetId>) {
        let n = y.dom().get(id).unwrap();
        if n.widget.type_name().ends_with("::EntryWidget") {
            ids.push(id);
        }
        for &c in &n.children {
            visit(y, c, ids);
        }
    }
    let mut ids = vec![];
    visit(y, y.dom().root(), &mut ids);
    ids
}
#[test]
fn select_aligns_the_current_choice_and_commits_stable_values_after_navigation() {
    let mut y = yak();
    let mut state = ui::SelectState::default();
    state.selected = Some(20);
    let (trigger, _) = frame(&mut y, &mut state, false, false);
    let anchor = y.layout_dom().get(trigger.id.unwrap()).unwrap().rect;
    click(&mut y, trigger.id.unwrap());
    frame(&mut y, &mut state, false, false);
    frame(&mut y, &mut state, false, false);
    let choices = entries(&y);
    let banana = y.layout_dom().get(choices[2]).unwrap().rect;
    assert!((banana.center().y - anchor.center().y).abs() < 0.01);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut state, false, false);
    assert_eq!(state.selected, Some(20));
    key(&mut y, KeyCode::Enter);
    assert!(frame(&mut y, &mut state, false, false).1);
    assert_eq!(state.selected, Some(30));
    assert!(!state.popup.open);
    assert!(!frame(&mut y, &mut state, false, false).1);
    frame(&mut y, &mut state, false, false);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut state, false, false);
    frame(&mut y, &mut state, false, false);
    key(&mut y, KeyCode::Home);
    key(&mut y, KeyCode::Enter);
    assert!(frame(&mut y, &mut state, false, false).1);
    assert_eq!(state.selected, None);
    assert!(!state.popup.open);
}
#[test]
fn select_compact_and_disabled_triggers_keep_their_size_and_cannot_open_when_disabled() {
    let mut y = yak();
    let mut state = ui::SelectState::default();
    let (trigger, _) = frame(&mut y, &mut state, true, true);
    assert_eq!(
        y.layout_dom().get(trigger.id.unwrap()).unwrap().rect.size(),
        Vec2::new(192., 28.)
    );
    click(&mut y, trigger.id.unwrap());
    frame(&mut y, &mut state, true, true);
    assert!(!state.popup.open);
    key(&mut y, KeyCode::Tab);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut state, true, true);
    assert!(!state.popup.open);
    frame(&mut y, &mut state, false, true);
    key(&mut y, KeyCode::Tab);
    frame(&mut y, &mut state, false, true);
    key(&mut y, KeyCode::ArrowDown);
    frame(&mut y, &mut state, false, true);
    frame(&mut y, &mut state, false, true);
    assert!(state.popup.open);
    key(&mut y, KeyCode::Escape);
    frame(&mut y, &mut state, false, true);
    assert!(!state.popup.open);
}

#[test]
fn select_tab_dismissal_moves_to_the_next_editable_field() {
    fn build(y: &mut Yakui, state: &mut ui::SelectState, text: &mut String) {
        y.start();
        let items = items();
        yakui::align(yakui::Alignment::TOP_LEFT, || {
            ui::width(192., || {
                ui::column(12., || {
                    ui::Select::new(&items).show(state);
                    ui::input(text, "Next field");
                })
            })
        });
        ui::SelectContent::new(&items)
            .align_item_with_trigger(false)
            .show(state);
        y.finish();
    }
    let mut y = yak();
    let mut state = ui::SelectState::default();
    let mut text = String::new();
    build(&mut y, &mut state, &mut text);
    key(&mut y, KeyCode::Tab);
    build(&mut y, &mut state, &mut text);
    key(&mut y, KeyCode::ArrowDown);
    build(&mut y, &mut state, &mut text);
    build(&mut y, &mut state, &mut text);
    key(&mut y, KeyCode::Tab);
    build(&mut y, &mut state, &mut text);
    build(&mut y, &mut state, &mut text);
    assert!(!state.popup.open);
    y.handle_event(Event::TextInput('q'));
    build(&mut y, &mut state, &mut text);
    assert_eq!(text, "q");
}
