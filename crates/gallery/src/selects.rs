//! The pinned English select previews, with ordinary caller-owned values.
use ui::SelectItem as O;
use yakui_shadcn as ui;
pub const VARIANTS: &[&str] = &[
    "default",
    "align-item",
    "groups",
    "scrollable",
    "disabled",
    "invalid",
];
pub fn items(variant: &str) -> Vec<O> {
    match variant {
        "groups" => vec![
            O::label("Fruits"),
            O::new(0, "Apple"),
            O::new(1, "Banana"),
            O::new(2, "Blueberry"),
            O::separator(),
            O::label("Vegetables"),
            O::new(3, "Carrot"),
            O::new(4, "Broccoli"),
            O::new(5, "Spinach"),
        ],
        "scrollable" => timezones(),
        _ => {
            let mut items = vec![];
            if variant == "default" {
                items.push(O::label("Fruits"));
            }
            items.push(O::empty("Select a fruit"));
            for (value, label) in ["Apple", "Banana", "Blueberry", "Grapes", "Pineapple"]
                .into_iter()
                .enumerate()
            {
                if variant == "invalid" && value > 2 {
                    break;
                }
                items.push(O::new(value, label).disabled(variant == "disabled" && value == 3));
            }
            items
        }
    }
}
pub fn render(variant: &str, state: &mut crate::examples::ExampleState, disabled: bool) {
    if state.select_variant != variant {
        state.select = Default::default();
        state.select_variant = variant.into();
        if variant == "align-item" {
            state.select.selected = Some(1);
            state.toggle = true;
        }
    }
    let items = items(variant);
    let mut trigger = || {
        ui::Select::new(&items)
            .placeholder(if variant == "scrollable" {
                "Select a timezone"
            } else {
                "Select a fruit"
            })
            .disabled(disabled || variant == "disabled")
            .invalid(variant == "invalid")
            .show(&mut state.select)
    };
    match variant {
        "align-item" => ui::width(320., || {
            ui::column(20., || {
                ui::Switch::new()
                    .label("Align Item")
                    .description("Toggle to align the item with the trigger.")
                    .trailing(true)
                    .show(&mut state.toggle);
                ui::fill_width(|| state.trigger = trigger());
            })
        }),
        "invalid" => ui::width(192., || {
            ui::column(8., || {
                let mut label_theme = (*ui::theme()).clone();
                label_theme.foreground = label_theme.destructive;
                ui::with_theme(label_theme, || ui::field_label("Fruit"));
                ui::fill_width(|| state.trigger = trigger());
                ui::field_error("Please select a fruit.");
            })
        }),
        _ => ui::width(if variant == "scrollable" { 256. } else { 192. }, || {
            state.trigger = trigger()
        }),
    }
}
pub fn content(variant: &str, state: &mut crate::examples::ExampleState) {
    let items = items(variant);
    if ui::SelectContent::new(&items)
        .align_item_with_trigger(variant != "align-item" || state.toggle)
        .show(&mut state.select)
    {
        state.count += 1;
    }
}
fn timezones() -> Vec<O> {
    let mut items = vec![];
    let mut value = 0;
    for (group, labels) in [
        (
            "North America",
            &[
                "Eastern Standard Time",
                "Central Standard Time",
                "Mountain Standard Time",
                "Pacific Standard Time",
                "Alaska Standard Time",
                "Hawaii Standard Time",
            ][..],
        ),
        (
            "Europe & Africa",
            &[
                "Greenwich Mean Time",
                "Central European Time",
                "Eastern European Time",
                "Western European Summer Time",
                "Central Africa Time",
                "East Africa Time",
            ][..],
        ),
        (
            "Asia",
            &[
                "Moscow Time",
                "India Standard Time",
                "China Standard Time",
                "Japan Standard Time",
                "Korea Standard Time",
                "Indonesia Central Standard Time",
            ][..],
        ),
        (
            "Australia & Pacific",
            &[
                "Australian Western Standard Time",
                "Australian Central Standard Time",
                "Australian Eastern Standard Time",
                "New Zealand Standard Time",
                "Fiji Time",
            ][..],
        ),
        (
            "South America",
            &[
                "Argentina Time",
                "Bolivia Time",
                "Brasilia Time",
                "Chile Standard Time",
            ][..],
        ),
    ] {
        items.push(O::label(group));
        for label in labels {
            items.push(O::new(value, *label));
            value += 1;
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use yakui::{Rect, Vec2, Yakui, event::Event, input::KeyCode};
    fn frame(y: &mut Yakui, state: &mut crate::examples::ExampleState) {
        y.start();
        yakui::center(|| render("scrollable", state, false));
        content("scrollable", state);
        y.finish();
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
    #[test]
    fn scrollable_select_example_reveals_and_selects_the_source_final_timezone() {
        let mut y = Yakui::new();
        let size = Vec2::new(1000., 800.);
        y.set_surface_size(size);
        y.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));
        ui::add_fonts(&mut y);
        let mut state = crate::examples::ExampleState::default();
        frame(&mut y, &mut state);
        key(&mut y, KeyCode::Tab);
        frame(&mut y, &mut state);
        key(&mut y, KeyCode::ArrowDown);
        frame(&mut y, &mut state);
        frame(&mut y, &mut state);
        assert!(state.select.popup.open);
        key(&mut y, KeyCode::End);
        frame(&mut y, &mut state);
        frame(&mut y, &mut state);
        // Enter uses the real focused row after native scrolling, not a direct
        // mutation of the value or a substitute list model.
        key(&mut y, KeyCode::Enter);
        frame(&mut y, &mut state);
        assert_eq!(state.select.selected, Some(26));
        assert_eq!(state.count, 1);
        assert!(!state.select.popup.open);
        frame(&mut y, &mut state);
        assert_eq!(state.count, 1);
    }
}
