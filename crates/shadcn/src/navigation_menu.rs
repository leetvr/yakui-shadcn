//! Native navigation flyouts composed from existing triggers, overlays and controls.
use crate::{ControlResponse, MenuState, Variant};
/// An application route or a trigger for richer navigation content.
#[derive(Debug, Clone)]
pub struct NavigationMenuEntry {
    pub id: u64,
    pub label: String,
    pub flyout: bool,
    pub disabled: bool,
}
impl NavigationMenuEntry {
    pub fn trigger(id: u64, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            flyout: true,
            disabled: false,
        }
    }
    pub fn link(id: u64, label: impl Into<String>) -> Self {
        Self {
            flyout: false,
            ..Self::trigger(id, label)
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}
/// App-owned navigation visibility. No URL or router is retained by the library.
#[derive(Debug, Default)]
pub struct NavigationMenuState {
    menus: Vec<MenuState>,
    active: Option<(usize, u64)>,
    keyboard_open: bool,
    hovered: Option<u64>,
}
impl NavigationMenuState {
    pub fn active(&self) -> Option<u64> {
        self.active
            .filter(|(index, _)| self.menus[*index].popup.open)
            .map(|(_, id)| id)
    }
    pub fn close(&mut self) {
        // Keep the last slot declared while closed: its FocusScope gets a layout
        // pass to return focus to the trigger. active() reports visibility.
        for menu in &mut self.menus {
            menu.close();
        }
    }
}
/// Draw navigation triggers and direct routes. Return a direct route's stable ID.
/// Render `navigation_menu_content` at the application root after this bar.
#[track_caller]
pub fn navigation_menu_bar(
    entries: &[NavigationMenuEntry],
    state: &mut NavigationMenuState,
) -> Option<u64> {
    state.menus.resize_with(entries.len(), MenuState::default);
    state.menus.truncate(entries.len());
    if state
        .active
        .is_some_and(|(index, _)| index >= entries.len())
    {
        state.active = None;
    }
    let mut route = None;
    let mut opened = None;
    let mut hovered = None;
    crate::row(0., || {
        for (index, entry) in entries.iter().enumerate() {
            let menu = &mut state.menus[index];
            let was_open = menu.popup.open;
            let button = || {
                crate::Button::new("")
                    .variant(if was_open {
                        Variant::Secondary
                    } else {
                        Variant::Ghost
                    })
                    .size(crate::ButtonSize::Lg)
                    .disabled(entry.disabled)
                    .show_children(|| {
                        crate::row(6., || {
                            let t = crate::theme();
                            yakui::widgets::RenderText::with_style(
                                &entry.label,
                                t.text_style(14., t.foreground, 500),
                            )
                            .show();
                            if entry.flyout {
                                crate::animation::animate(
                                    if was_open { std::f32::consts::PI } else { 0. },
                                    crate::animation::Motion::new(
                                        std::time::Duration::from_millis(300),
                                        crate::animation::Easing::Standard,
                                    ),
                                    |rotation| {
                                        crate::animation::motion(
                                            crate::animation::MotionStyle {
                                                rotation,
                                                ..Default::default()
                                            },
                                            || crate::icon(crate::Icon::ChevronDown, 12.),
                                        );
                                    },
                                );
                            }
                        })
                    })
            };
            if entry.flyout {
                let response = crate::dropdown_menu_trigger(menu, button);
                if response.hovering {
                    hovered = Some(entry.id);
                }
                if response.hovering && !entry.disabled && state.hovered != Some(entry.id) {
                    menu.open_from(response);
                    state.keyboard_open = false;
                }
                if menu.popup.open && (!was_open || response.clicked || response.hovering) {
                    opened = Some((index, entry.id));
                    if !response.hovering {
                        state.keyboard_open = response.focus_visible;
                    }
                }
            } else if button().clicked {
                route = Some(entry.id);
            }
        }
    });
    state.hovered = hovered;
    if let Some(active) = opened {
        state.active = Some(active);
    }
    if let Some((active, _)) = state.active {
        for (index, menu) in state.menus.iter_mut().enumerate() {
            if index != active {
                menu.close();
            }
        }
    }
    if route.is_some() {
        state.close();
    }
    route
}
/// Render the active flyout. A returned link/action ID closes it and restores focus.
#[track_caller]
pub fn navigation_menu_content(
    state: &mut NavigationMenuState,
    width: f32,
    children: impl FnOnce(u64) -> Option<u64>,
) -> Option<u64> {
    let Some((index, id)) = state.active else {
        return None;
    };
    let mut action = None;
    crate::overlay::navigation_popup(
        &mut state.menus[index].popup,
        width,
        state.keyboard_open,
        || {
            action = children(id);
        },
    );
    if action.is_some() || !state.menus[index].popup.open {
        state.close();
    }
    action
}
/// A focusable navigation link, optionally containing a description and icon.
#[derive(Debug, Clone)]
pub struct NavigationMenuLink {
    title: String,
    description: Option<String>,
    description_lines: Option<usize>,
    icon: Option<crate::Icon>,
    active: bool,
    disabled: bool,
}
impl NavigationMenuLink {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            description_lines: None,
            icon: None,
            active: false,
            disabled: false,
        }
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    /// Clip long descriptions to this many body lines.
    pub fn description_lines(mut self, lines: usize) -> Self {
        self.description_lines = Some(lines);
        self
    }
    pub fn icon(mut self, icon: crate::Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    #[track_caller]
    pub fn show(self) -> ControlResponse {
        let t = crate::theme();
        crate::interaction::control(
            crate::interaction::Control {
                disabled: self.disabled,
                radius: t.radius * 0.8,
                ring: t.ring,
                focus_size: None,
            },
            |response| {
                crate::fill_width(|| {
                    crate::surface(
                        if response.hovering || response.focused || self.active {
                            t.muted
                        } else {
                            crate::Color::CLEAR
                        },
                        None,
                        t.radius * 0.8,
                        8.,
                        || {
                            if let Some(description) = self.description {
                                crate::column(4., || {
                                    crate::label(self.title);
                                    if let Some(lines) = self.description_lines {
                                        yakui::widgets::ConstrainedBox::new(yakui::Constraints {
                                            min: crate::Vec2::ZERO,
                                            max: crate::Vec2::new(
                                                f32::INFINITY,
                                                t.line_height * lines as f32,
                                            ),
                                        })
                                        .show(|| {
                                            crate::clip(|| crate::muted(description));
                                        });
                                    } else {
                                        crate::muted(description);
                                    }
                                });
                            } else {
                                crate::row(8., || {
                                    if let Some(icon) = self.icon {
                                        crate::icon_colored(icon, 16., t.muted_foreground);
                                    }
                                    crate::text(self.title);
                                });
                            }
                        },
                    );
                });
            },
        )
    }
}
