//! Pinned informational compositions; all targets/surfaces come from the library.
use yakui_adequate_ui as ui;
pub const VARIANTS: &[&str] = &["default", "sides", "keyboard", "disabled"];
pub fn render(variant: &str, state: &mut crate::examples::ExampleState, disabled: bool) {
    match variant {
        "sides" => ui::row(8., || {
            for (index, label) in ["Left", "Top", "Bottom", "Right"].iter().enumerate() {
                let trigger = ui::tooltip_trigger(&mut state.tooltips[index], || {
                    ui::Button::new(*label)
                        .variant(ui::Variant::Outline)
                        .disabled(disabled)
                        .show()
                });
                if index == 0 {
                    state.trigger = trigger;
                }
            }
        }),
        "disabled" => {
            state.trigger = ui::tooltip_trigger_area(&mut state.tooltips[0], || {
                ui::Button::new("Disabled")
                    .variant(ui::Variant::Outline)
                    .disabled(true)
                    .show();
            })
        }
        "keyboard" => {
            state.trigger = ui::tooltip_trigger(&mut state.tooltips[0], || {
                ui::Button::new("")
                    .variant(ui::Variant::Outline)
                    .size(ui::ButtonSize::IconSm)
                    .icon(ui::Icon::Save)
                    .disabled(disabled)
                    .show()
            })
        }
        _ => {
            state.trigger = ui::tooltip_trigger(&mut state.tooltips[0], || {
                ui::Button::new("Hover")
                    .variant(ui::Variant::Outline)
                    .disabled(disabled)
                    .show()
            })
        }
    }
}
pub fn content(variant: &str, state: &mut crate::examples::ExampleState) {
    if variant == "sides" {
        for (index, side) in [
            ui::Side::Left,
            ui::Side::Top,
            ui::Side::Bottom,
            ui::Side::Right,
        ]
        .iter()
        .enumerate()
        {
            ui::Tooltip::new()
                .side(*side)
                .show(&mut state.tooltips[index], "Add to library");
        }
    } else if variant == "keyboard" {
        ui::Tooltip::new().show_children(&mut state.tooltips[0], || {
            ui::text("Save Changes");
            ui::kbd("S");
        });
    } else {
        ui::Tooltip::new().show(
            &mut state.tooltips[0],
            if variant == "disabled" {
                "This feature is currently unavailable"
            } else {
                "Add to library"
            },
        );
    }
}
