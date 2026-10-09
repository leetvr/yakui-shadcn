//! Checkbox fields share activation across the indicator, label and helper text.
use crate::{
    Color, ControlResponse, Icon, Vec2,
    interaction::{Control, control},
};
use yakui::{
    Border, Constraints, Rect,
    widget::{LayoutContext, PaintContext, Widget},
    widgets::{RenderText, RoundRect},
};

/// A bare checkbox or a labelled field, backed by a caller-owned boolean.
///
/// ```no_run
/// # use yakui_adequate_ui as ui;
/// # let mut enabled = false;
/// ui::Checkbox::new()
///     .label("Enable notifications")
///     .description("You can change this at any time.")
///     .card(true)
///     .show(&mut enabled);
/// ```
/// The entire field activates the checkbox and occupies one keyboard focus stop.
/// Disabled fields ignore input. Invalid fields retain normal activation behaviour.
#[derive(Debug, Clone, Default)]
pub struct Checkbox {
    label: Option<String>,
    description: Option<String>,
    disabled: bool,
    invalid: bool,
    card: bool,
    label_weight: Option<u16>,
}
impl Checkbox {
    pub fn new() -> Self {
        Self::default()
    }
    /// Associate visible text with the indicator's native hit target.
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }
    /// Helper text below the label; wraps within the available field width.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
    /// Render the upstream FieldLabel card composition, including selected styling.
    pub fn card(mut self, card: bool) -> Self {
        self.card = card;
        self
    }
    /// Set the label's font weight (400 for ordinary checkbox lists; default 500).
    pub fn label_weight(mut self, weight: u16) -> Self {
        self.label_weight = Some(weight);
        self
    }
    #[track_caller]
    pub fn show(self, value: &mut bool) -> ControlResponse {
        let t = crate::theme();
        let background = crate::layout::surface_background();
        control(
            Control {
                disabled: self.disabled,
                radius: t.radius * 0.8,
                ring: if self.card { t.ring } else { Color::CLEAR },
                focus_size: None,
            },
            |response| {
                if response.clicked {
                    *value = !*value;
                }
                let body = || {
                    let has_description = self.description.as_ref().is_some_and(|s| !s.is_empty());
                    yakui::util::widget_children::<CheckboxLayout, _>(
                        || {
                            let base = crate::layout::surface_background();
                            let mut fill = if *value {
                                t.primary
                            } else if t.dark {
                                crate::layout::composite(t.input.with_alpha(0.045), base)
                            } else {
                                base
                            };
                            let mut border = if *value {
                                t.primary
                            } else {
                                crate::layout::resolve_color(t.input)
                            };
                            if self.invalid && !*value {
                                border = crate::layout::resolve_color(
                                    t.destructive.with_alpha(if t.dark { 0.5 } else { 1. }),
                                );
                            }
                            if response.focus_visible && !self.card && !self.invalid {
                                border = t.ring;
                            }
                            let mut foreground = t.primary_foreground;
                            if self.disabled {
                                fill = crate::layout::composite(fill.with_alpha(0.5), base);
                                border = crate::layout::composite(border.with_alpha(0.5), base);
                                foreground =
                                    crate::layout::composite(foreground.with_alpha(0.5), base);
                            }
                            let ring =
                                if self.disabled || self.card {
                                    Color::CLEAR
                                } else if self.invalid {
                                    crate::layout::resolve_color(
                                        t.destructive.with_alpha(if t.dark { 0.4 } else { 0.2 }),
                                    )
                                } else if response.focus_visible {
                                    crate::layout::resolve_color(t.ring.with_alpha(0.5))
                                } else {
                                    Color::CLEAR
                                };
                            yakui::util::widget_children::<IndicatorRing, _>(
                                || {
                                    crate::size(Vec2::splat(16.), || {
                                        RoundRect::new(4.)
                                            .color(fill)
                                            .border(Some(Border::new(border, 1.)))
                                            .show_children(|| {
                                                if *value {
                                                    crate::min_size(Vec2::ZERO, || {
                                                        crate::icon_colored(
                                                            Icon::Check,
                                                            14.,
                                                            foreground,
                                                        )
                                                    });
                                                }
                                            });
                                    });
                                },
                                ring,
                            );
                            if let Some(label) = &self.label {
                                crate::column(2., || {
                                    let fg = if self.invalid {
                                        t.destructive
                                    } else {
                                        t.foreground
                                    };
                                    let fg = crate::layout::resolve_color(
                                        fg.with_alpha(if self.disabled { 0.5 } else { 1. }),
                                    );
                                    let mut style =
                                        t.text_style(14., fg, self.label_weight.unwrap_or(500));
                                    style.line_height_override =
                                        Some(if self.card { 19.25 } else { 14. });
                                    RenderText::with_style(label.clone(), style).show();
                                    if has_description {
                                        let mut style = t.text_style(14., t.muted_foreground, 400);
                                        style.line_height_override = Some(21.);
                                        RenderText::with_style(
                                            self.description.clone().unwrap(),
                                            style,
                                        )
                                        .show();
                                    }
                                });
                            }
                        },
                        (has_description, crate::advanced_layout::rtl()),
                    );
                };
                if self.card {
                    let fill = if *value {
                        t.primary.with_alpha(if t.dark { 0.1 } else { 0.05 })
                    } else if response.hovering && !self.disabled {
                        t.muted.with_alpha(0.5)
                    } else {
                        background
                    };
                    let border = if response.focus_visible {
                        t.ring
                    } else if *value {
                        t.primary.with_alpha(if t.dark { 0.2 } else { 0.3 })
                    } else {
                        t.border
                    };
                    crate::fill_width(|| {
                        crate::surface(fill, Some(border), t.radius * 0.8, 11., body)
                    });
                } else {
                    body();
                }
            },
        )
    }
}

#[derive(Debug, Default)]
struct CheckboxLayout {
    description: bool,
    rtl: bool,
}
impl Widget for CheckboxLayout {
    type Props<'a> = (bool, bool);
    type Response = ();
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, (description, rtl): Self::Props<'_>) {
        self.description = description;
        self.rtl = rtl;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let children = &ctx.dom.get_current().children;
        let indicator = children[0];
        let indicator_size = ctx.calculate_layout(indicator, Constraints::loose(constraints.max));
        let mut content = Vec2::ZERO;
        if let Some(&label) = children.get(1) {
            content = ctx.calculate_layout(
                label,
                Constraints::loose(Vec2::new(
                    (constraints.max.x - indicator_size.x - 8.).max(0.),
                    constraints.max.y,
                )),
            );
        }
        let size = constraints.constrain(Vec2::new(
            indicator_size.x
                + if children.len() > 1 {
                    8. + content.x
                } else {
                    0.
                },
            indicator_size.y.max(content.y),
        ));
        // Helper text may be several lines tall. The indicator stays at the first
        // line, and its own paint node owns the ring so RTL and wrapping cannot
        // move keyboard focus decoration into the middle of the entire field.
        let y = if self.description {
            1.
        } else {
            (size.y - indicator_size.y) / 2.
        };
        ctx.layout.set_pos(
            indicator,
            Vec2::new(
                if self.rtl {
                    size.x - indicator_size.x
                } else {
                    0.
                },
                y,
            ),
        );
        if let Some(&label) = children.get(1) {
            ctx.layout.set_pos(
                label,
                Vec2::new(
                    if self.rtl {
                        size.x - indicator_size.x - 8. - content.x
                    } else {
                        indicator_size.x + 8.
                    },
                    0.,
                ),
            );
        }
        size
    }
}
#[derive(Debug)]
struct IndicatorRing(Color);
impl Widget for IndicatorRing {
    type Props<'a> = Color;
    type Response = ();
    fn new() -> Self {
        Self(Color::CLEAR)
    }
    fn update(&mut self, color: Color) {
        self.0 = color;
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        if self.0.a != 0 {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            crate::icons::outline(
                &mut ctx,
                Rect::from_pos_size(rect.pos() - Vec2::splat(3.), rect.size() + Vec2::splat(6.)),
                7.,
                3.,
                self.0,
            );
        }
        for &child in &ctx.dom.get_current().children {
            ctx.paint(child);
        }
    }
}
