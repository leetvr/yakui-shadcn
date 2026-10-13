//! Caller-owned switch values, labels, helper text and choice-card composition.
use crate::{
    Color, ControlResponse, Vec2,
    control_field::{FieldControlLayout, FieldControlLayoutProps, IndicatorRing},
    interaction::{Control, control},
};
use yakui::{
    Border,
    widgets::{Pad, RenderText, RoundRect},
};
/// The two documented switch sizes.
#[derive(Debug, Clone, Copy, Default)]
pub enum SwitchSize {
    #[default]
    Default,
    Sm,
}
impl SwitchSize {
    pub fn dimensions(self) -> Vec2 {
        match self {
            Self::Default => Vec2::new(32., 18.4),
            Self::Sm => Vec2::new(24., 14.),
        }
    }
    fn thumb(self) -> f32 {
        match self {
            Self::Default => 16.,
            Self::Sm => 12.,
        }
    }
}
/// A bare switch or a labelled field with a single native activation/focus target.
/// Labels and helper text activate the same caller-owned boolean as the track.
///
/// ```no_run
/// # use yakui_shadcn as ui;
/// # let mut shared = false;
/// ui::width(384., || {
///     ui::Switch::new().label("Share across devices")
///         .description("Focus is shared across devices.")
///         .trailing(true).card(true).show(&mut shared);
/// });
/// ```
#[derive(Debug, Clone, Default)]
pub struct Switch {
    label: Option<String>,
    description: Option<String>,
    size: SwitchSize,
    disabled: bool,
    invalid: bool,
    card: bool,
    trailing: bool,
}
impl Switch {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
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
    /// Render the track after the field content, mirroring it in RTL scopes.
    pub fn trailing(mut self, trailing: bool) -> Self {
        self.trailing = trailing;
        self
    }
    /// Selectable FieldLabel card, with a visible selected/hover/focus surface.
    pub fn card(mut self, card: bool) -> Self {
        self.card = card;
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
                    let description = self.description.as_ref().is_some_and(|s| !s.is_empty());
                    let rtl = crate::advanced_layout::rtl();
                    yakui::util::widget_children::<FieldControlLayout, _>(
                        || {
                            let base = crate::layout::surface_background();
                            let mut fill = if *value {
                                t.primary
                            } else {
                                crate::layout::composite(
                                    t.input.with_alpha(if t.dark { 0.12 } else { 1. }),
                                    base,
                                )
                            };
                            let mut thumb_color = if *value {
                                t.primary_foreground
                            } else if t.dark {
                                t.foreground
                            } else {
                                t.background
                            };
                            let mut border = if self.invalid {
                                Some(crate::layout::resolve_color(
                                    t.destructive.with_alpha(if t.dark { 0.5 } else { 1. }),
                                ))
                            } else if response.focus_visible && !self.card {
                                Some(t.ring)
                            } else {
                                None
                            };
                            if self.disabled {
                                fill = crate::layout::composite(fill.with_alpha(0.5), base);
                                thumb_color =
                                    crate::layout::composite(thumb_color.with_alpha(0.5), base);
                                border = border.map(|color| {
                                    crate::layout::composite(color.with_alpha(0.5), base)
                                });
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
                            let dimensions = self.size.dimensions();
                            let thumb = self.size.thumb();
                            let thumb_x = if *value != rtl {
                                dimensions.x - thumb - 1.
                            } else {
                                1.
                            };
                            crate::animation::animate(
                                (fill, thumb_color, thumb_x),
                                Default::default(),
                                |(fill, thumb_color, thumb_x)| {
                                    yakui::util::widget_children::<IndicatorRing, _>(
                                        || {
                                            crate::size(dimensions, || {
                                                RoundRect::new(999.)
                                                    .color(fill)
                                                    .border(
                                                        border.map(|c| {
                                                            Border::new(c, t.border_width)
                                                        }),
                                                    )
                                                    .show_children(|| {
                                                        yakui::align(
                                                            yakui::Alignment::TOP_LEFT,
                                                            || {
                                                                yakui::pad(
                                                                    Pad {
                                                                        left: thumb_x,
                                                                        right: 0.,
                                                                        top: (dimensions.y - thumb)
                                                                            / 2.,
                                                                        bottom: 0.,
                                                                    },
                                                                    || {
                                                                        crate::size(
                                                                            Vec2::splat(thumb),
                                                                            || {
                                                                                RoundRect::new(
                                                                                    999.,
                                                                                )
                                                                                .color(thumb_color)
                                                                                .show();
                                                                            },
                                                                        )
                                                                    },
                                                                );
                                                            },
                                                        );
                                                    });
                                            });
                                        },
                                        (ring, 999.),
                                    );
                                },
                            );
                            if let Some(label) = &self.label {
                                crate::column(2., || {
                                    let color = if self.invalid {
                                        t.destructive
                                    } else {
                                        t.foreground
                                    };
                                    let color = crate::layout::resolve_color(
                                        color.with_alpha(if self.disabled { 0.5 } else { 1. }),
                                    );
                                    let mut style = t.text_style(14., color, 500);
                                    style.line_height_override =
                                        Some(if self.card { 19.25 } else { 14. });
                                    RenderText::with_style(label.clone(), style).show();
                                    if description {
                                        crate::field_description(self.description.clone().unwrap());
                                    }
                                });
                            }
                        },
                        FieldControlLayoutProps {
                            description,
                            rtl,
                            trailing: self.trailing,
                            indicator_top: 0.,
                        },
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
