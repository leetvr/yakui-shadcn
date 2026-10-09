//! Typography, actions, content surfaces and form controls.
use crate::interaction::{Control, control};
use crate::{ControlResponse, Icon, column, icon_colored, row, surface, theme};
use yakui::{
    Border, Color, Constraints, Vec2,
    widgets::{ConstrainedBox, Pad, RenderText, RoundRect},
};

/// Draw body text using the scoped theme.
#[track_caller]
pub fn text(text: impl Into<String>) {
    let t = theme();
    RenderText::with_style(text, t.text_style(t.font_size, t.foreground, 400)).show();
}
/// Draw secondary explanatory text.
#[track_caller]
pub fn muted(text: impl Into<String>) {
    let t = theme();
    RenderText::with_style(text, t.text_style(t.font_size, t.muted_foreground, 400)).show();
}
/// Draw a medium-weight form label.
#[track_caller]
pub fn label(text: impl Into<String>) {
    let t = theme();
    RenderText::with_style(text, t.text_style(t.font_size, t.foreground, 500)).show();
}
/// Draw a heading (levels 1 through 4).
#[track_caller]
pub fn heading(level: u8, text: impl Into<String>) {
    let t = theme();
    let size = match level {
        1 => 36.,
        2 => 30.,
        3 => 24.,
        _ => 20.,
    };
    RenderText::with_style(text, t.text_style(size, t.foreground, 600)).show();
}
/// Shared shadcn action/badge variant vocabulary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Variant {
    #[default]
    Default,
    Outline,
    Secondary,
    Ghost,
    Destructive,
    Link,
}
/// All eight upstream button sizes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Default,
    Xs,
    Sm,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}
impl ButtonSize {
    pub fn height(self) -> f32 {
        match self {
            Self::Xs | Self::IconXs => 24.,
            Self::Sm | Self::IconSm => 28.,
            Self::Lg | Self::IconLg => 36.,
            _ => 32.,
        }
    }
    fn icon(self) -> bool {
        matches!(
            self,
            Self::Icon | Self::IconXs | Self::IconSm | Self::IconLg
        )
    }
    fn font_size(self) -> f32 {
        match self {
            Self::Xs | Self::IconXs => 12.,
            Self::Sm | Self::IconSm => 12.8,
            _ => 14.,
        }
    }
}
/// A native focusable action. Enter activates on press; Space activates on release.
#[derive(Debug, Clone)]
pub struct Button {
    text: String,
    variant: Variant,
    size: ButtonSize,
    disabled: bool,
    icon: Option<Icon>,
    rounded: bool,
}
impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: Variant::Default,
            size: ButtonSize::Default,
            disabled: false,
            icon: None,
            rounded: false,
        }
    }
    pub fn variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }
    #[track_caller]
    pub fn show(self) -> ControlResponse {
        let t = theme();
        let radius = if self.rounded {
            999.
        } else {
            if matches!(
                self.size,
                ButtonSize::Xs | ButtonSize::Sm | ButtonSize::IconXs | ButtonSize::IconSm
            ) {
                t.radius * 0.8
            } else {
                t.radius
            }
        };
        control(
            Control {
                disabled: self.disabled,
                radius,
                ring: t.ring,
            },
            |res| {
                let active = res.hovering || res.pressed;
                let (mut fill, mut fg, mut border) = match self.variant {
                    Variant::Default => (
                        if active {
                            t.primary.with_alpha(0.8)
                        } else {
                            t.primary
                        },
                        t.primary_foreground,
                        None,
                    ),
                    Variant::Outline => (
                        if active {
                            t.muted
                        } else if t.dark {
                            t.input.with_alpha(0.045)
                        } else {
                            t.background
                        },
                        t.foreground,
                        Some(t.border),
                    ),
                    Variant::Secondary => (
                        if active {
                            t.secondary.lerp(&t.foreground, 0.05)
                        } else {
                            t.secondary
                        },
                        t.secondary_foreground,
                        None,
                    ),
                    Variant::Ghost => (
                        if active { t.muted } else { Color::CLEAR },
                        t.foreground,
                        None,
                    ),
                    Variant::Destructive => (
                        t.destructive.with_alpha(if active {
                            if t.dark { 0.3 } else { 0.2 }
                        } else if t.dark {
                            0.2
                        } else {
                            0.1
                        }),
                        t.destructive,
                        None,
                    ),
                    Variant::Link => (Color::CLEAR, t.primary, None),
                };
                let background = crate::layout::surface_background();
                fill = crate::layout::composite(fill, background);
                border = border.map(|c| crate::layout::composite(c, fill));
                if self.disabled {
                    fill = crate::layout::composite(fill.with_alpha(0.5), background);
                    fg = crate::layout::composite(fg.with_alpha(0.5), background);
                    border =
                        border.map(|c| crate::layout::composite(c.with_alpha(0.5), background));
                }
                let height = self.size.height();
                let fs = self.size.font_size();
                let line_height = if fs < 14. { 16. } else { 20. };
                let horizontal = 1.0
                    + if matches!(self.size, ButtonSize::Xs) {
                        8.
                    } else {
                        10.
                    };
                let min_width = if self.size.icon() { height } else { 0. };
                ConstrainedBox::new(Constraints {
                    min: Vec2::new(min_width, height),
                    max: Vec2::new(
                        if self.size.icon() {
                            height
                        } else {
                            f32::INFINITY
                        },
                        height,
                    ),
                })
                .show(|| {
                    RoundRect::new(radius)
                        .color(fill)
                        .border(border.map(|c| Border::new(c, t.border_width)))
                        .show_children(|| {
                            yakui::pad(
                                Pad::balanced(
                                    if self.size.icon() { 0. } else { horizontal },
                                    (height - line_height) / 2.,
                                ),
                                || {
                                    row(6., || {
                                        if let Some(icon) = self.icon {
                                            icon_colored(
                                                icon,
                                                if height == 24. { 12. } else { 16. },
                                                fg,
                                            );
                                        }
                                        if !self.size.icon() {
                                            let mut style = t.text_style(fs, fg, 500);
                                            style.line_height_override = Some(line_height);
                                            RenderText::with_style(self.text, style).show();
                                        }
                                    });
                                },
                            );
                        });
                });
            },
        )
    }
}
/// The default action shorthand.
#[track_caller]
pub fn button(text: impl Into<String>) -> ControlResponse {
    Button::new(text).show()
}
/// A compact status label, with all six upstream variants.
#[track_caller]
pub fn badge(text: impl Into<String>, variant: Variant) {
    let t = theme();
    let (fill, fg, border) = match variant {
        Variant::Default => (t.primary, t.primary_foreground, None),
        Variant::Secondary => (t.secondary, t.secondary_foreground, None),
        Variant::Outline => (Color::CLEAR, t.foreground, Some(t.border)),
        Variant::Destructive => (
            t.destructive.with_alpha(if t.dark { 0.2 } else { 0.1 }),
            t.destructive,
            None,
        ),
        Variant::Ghost => (Color::CLEAR, t.foreground, None),
        Variant::Link => (Color::CLEAR, t.primary, None),
    };
    RoundRect::new(999.)
        .color(fill)
        .border(border.map(|c| Border::new(c, 1.)))
        .show_children(|| {
            yakui::pad(Pad::balanced(8., 2.), || {
                let mut style = t.text_style(12., fg, 500);
                style.line_height_override = Some(16.);
                RenderText::with_style(text, style).show();
            });
        });
}
/// Card size controls padding (16px default, 12px small).
#[derive(Debug, Default, Clone, Copy)]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}
#[derive(Debug, Clone, Copy)]
struct CardSpacing(f32);
/// A composable content surface; use the header/content/footer slots inside it.
#[track_caller]
pub fn card(children: impl FnOnce()) {
    card_sized(CardSize::Default, children)
}
#[track_caller]
pub fn card_sized(size: CardSize, children: impl FnOnce()) {
    let t = theme();
    let spacing = match size {
        CardSize::Default => 16.,
        CardSize::Sm => 12.,
    };
    surface(
        t.card,
        Some(t.foreground.with_alpha(0.1)),
        t.radius * 1.4,
        0.,
        || {
            yakui::widgets::Scope::new(CardSpacing(spacing)).show(|| column(spacing, children));
        },
    );
}
fn card_spacing() -> f32 {
    yakui::util::read_scope::<CardSpacing>().map_or(16., |s| s.0)
}
#[track_caller]
pub fn card_header(children: impl FnOnce()) {
    yakui::pad(Pad::all(card_spacing()), || column(4., children));
}
#[track_caller]
pub fn card_title(text: impl Into<String>) {
    let t = theme();
    RenderText::with_style(
        text,
        t.text_style(
            if card_spacing() == 12. { 14. } else { 16. },
            t.card_foreground,
            500,
        ),
    )
    .show();
}
#[track_caller]
pub fn card_description(text: impl Into<String>) {
    muted(text)
}
#[track_caller]
pub fn card_content(children: impl FnOnce()) {
    yakui::pad(Pad::balanced(card_spacing(), 0.), children);
}
#[track_caller]
pub fn card_footer(children: impl FnOnce()) {
    let t = theme();
    surface(
        t.muted.with_alpha(0.5),
        Some(t.border),
        t.radius * 1.4,
        card_spacing(),
        children,
    );
}
/// Thin horizontal separator with an explicit width.
#[track_caller]
pub fn separator(width: f32) {
    crate::size(Vec2::new(width, theme().border_width), || {
        RoundRect::new(0.).color(theme().border).show();
    });
}
/// Muted placeholder surface. Animation is driven by the application frame clock.
#[track_caller]
pub fn skeleton(size: Vec2) {
    crate::size(size, || {
        RoundRect::new(theme().radius - 2.)
            .color(theme().muted)
            .show();
    });
}
/// Bounded progress in the range 0..=1; non-finite values are treated as zero.
#[track_caller]
pub fn progress(value: f32, width: f32) {
    let t = theme();
    let value = if value.is_finite() {
        value.clamp(0., 1.)
    } else {
        0.
    };
    crate::size(Vec2::new(width, 6.), || {
        RoundRect::new(999.)
            .color(t.primary.with_alpha(0.2))
            .show_children(|| {
                crate::size(Vec2::new(width * value, 6.), || {
                    RoundRect::new(999.).color(t.primary).show();
                });
            });
    });
}
/// Keyboard shortcut hint.
#[track_caller]
pub fn kbd(value: impl Into<String>) {
    let t = theme();
    surface(t.muted, None, 4., 2., || {
        RenderText::with_style(value, t.text_style(12., t.muted_foreground, 500)).show();
    });
}
/// Avatar fallback. Image avatars accept Yakui texture IDs through normal composition.
#[track_caller]
pub fn avatar(initials: impl Into<String>, size: f32) {
    let t = theme();
    crate::size(Vec2::splat(size), || {
        RoundRect::new(999.).color(t.muted).show_children(|| {
            yakui::center(|| {
                RenderText::with_style(initials, t.text_style(14., t.muted_foreground, 500)).show();
            });
        });
    });
}
/// Alert surface with title and description.
#[track_caller]
pub fn alert(title: impl Into<String>, description: impl Into<String>, destructive: bool) {
    let t = theme();
    surface(t.card, Some(t.border), t.radius, 10., || {
        column(2., || {
            RenderText::with_style(
                title,
                t.text_style(
                    14.,
                    if destructive {
                        t.destructive
                    } else {
                        t.card_foreground
                    },
                    500,
                ),
            )
            .show();
            RenderText::with_style(
                description,
                t.text_style(
                    14.,
                    if destructive {
                        t.destructive
                    } else {
                        t.muted_foreground
                    },
                    400,
                ),
            )
            .show();
        })
    });
}
/// Placeholder content with a title and description, composed in the public library.
#[track_caller]
pub fn empty(title: impl Into<String>, description: impl Into<String>, children: impl FnOnce()) {
    surface(
        Color::CLEAR,
        Some(theme().border),
        theme().radius,
        24.,
        || {
            column(12., || {
                heading(4, title);
                muted(description);
                children();
            })
        },
    );
}
/// Label, control, description and optional error. Validation stays application-owned.
#[track_caller]
pub fn field(
    title: impl Into<String>,
    description: impl Into<String>,
    error: Option<&str>,
    children: impl FnOnce(),
) {
    let description = description.into();
    column(6., || {
        label(title);
        crate::fill_width(children);
        if !description.is_empty() {
            muted(description);
        }
        if let Some(error) = error {
            let t = theme();
            RenderText::with_style(error, t.text_style(14., t.destructive, 400)).show();
        }
    });
}
/// Edited text is copied back to application state; `true` reports a text change.
#[track_caller]
pub fn input(value: &mut String, placeholder: &str) -> bool {
    input_multiline(value, placeholder, false)
}
/// A multiline editor using Yakui's Cosmic Text selection, clipboard and cursor.
#[track_caller]
pub fn textarea(value: &mut String, placeholder: &str) -> bool {
    input_multiline(value, placeholder, true)
}
#[track_caller]
fn input_multiline(value: &mut String, placeholder: &str, multiline: bool) -> bool {
    let t = theme();
    let mut changed = false;
    surface(
        if t.dark {
            t.input.with_alpha(0.045)
        } else {
            Color::CLEAR
        },
        Some(t.input),
        t.radius,
        0.,
        || {
            let editor = yakui::widgets::TextBox::new(value.clone())
                .style(t.text_style(t.font_size, t.foreground, 400))
                .padding(Pad::balanced(10., 6.))
                .fill(None)
                .radius(t.radius)
                .placeholder(placeholder.to_owned())
                .cursor_color(t.foreground)
                .selection_halo_color(Color::CLEAR)
                .selected_bg_color(t.accent)
                .multiline(multiline)
                .inline_edit(!multiline);
            let res = yakui::util::widget::<crate::editor::Editor>(editor);
            if let Some(next) = res.into_inner().text {
                *value = next;
                changed = true;
            }
        },
    );
    changed
}
/// Checkbox and label share one hit target and keyboard focus.
#[track_caller]
pub fn checkbox(label_text: impl Into<String>, value: &mut bool) -> ControlResponse {
    checkbox_disabled(label_text, value, false)
}
#[track_caller]
pub fn checkbox_disabled(
    label_text: impl Into<String>,
    value: &mut bool,
    disabled: bool,
) -> ControlResponse {
    let t = theme();
    let res = control(
        Control {
            disabled,
            radius: 4.,
            ring: t.ring,
        },
        |res| {
            if res.clicked {
                *value = !*value;
            }
            row(8., || {
                crate::size(Vec2::splat(16.), || {
                    RoundRect::new(4.)
                        .color(if *value { t.primary } else { Color::CLEAR })
                        .border(Some(Border::new(
                            if *value { t.primary } else { t.input },
                            1.,
                        )))
                        .show_children(|| {
                            if *value {
                                icon_colored(Icon::Check, 14., t.primary_foreground);
                            }
                        });
                });
                RenderText::with_style(
                    label_text,
                    t.text_style(
                        14.,
                        t.foreground.with_alpha(if disabled { 0.5 } else { 1. }),
                        500,
                    ),
                )
                .show();
            });
        },
    );
    res
}
/// A two-state compact control, with explicit disabled state.
#[track_caller]
pub fn switch(label_text: impl Into<String>, value: &mut bool, disabled: bool) -> ControlResponse {
    let t = theme();
    control(
        Control {
            disabled,
            radius: 999.,
            ring: t.ring,
        },
        |res| {
            if res.clicked {
                *value = !*value;
            }
            row(8., || {
                crate::size(Vec2::new(28., 16.), || {
                    RoundRect::new(999.)
                        .color(if *value { t.primary } else { t.input })
                        .show_children(|| {
                            yakui::pad(
                                Pad {
                                    left: if *value { 13. } else { 1. },
                                    right: 0.,
                                    top: 1.,
                                    bottom: 1.,
                                },
                                || {
                                    crate::size(Vec2::splat(14.), || {
                                        RoundRect::new(999.)
                                            .color(if *value {
                                                t.primary_foreground
                                            } else {
                                                t.background
                                            })
                                            .show();
                                    });
                                },
                            );
                        });
                });
                label(label_text);
            });
        },
    )
}
/// A persistent pressed action, useful for formatting toolbars.
#[track_caller]
pub fn toggle(text: impl Into<String>, value: &mut bool, outline: bool) -> ControlResponse {
    let res = Button::new(text)
        .variant(if *value {
            Variant::Secondary
        } else if outline {
            Variant::Outline
        } else {
            Variant::Ghost
        })
        .show();
    if res.clicked {
        *value = !*value;
    }
    res
}
/// Single-selection radio group. Directional navigation is handled by Yakui.
#[track_caller]
pub fn radio_group(options: &[&str], selected: &mut usize) {
    let t = theme();
    column(10., || {
        for (i, option) in options.iter().enumerate() {
            control(
                Control {
                    disabled: false,
                    radius: 999.,
                    ring: t.ring,
                },
                |res| {
                    if res.clicked {
                        *selected = i;
                    }
                    row(8., || {
                        crate::size(Vec2::splat(16.), || {
                            RoundRect::new(999.)
                                .color(Color::CLEAR)
                                .border(Some(Border::new(t.input, 1.)))
                                .show_children(|| {
                                    if *selected == i {
                                        yakui::pad(Pad::all(4.), || {
                                            crate::size(Vec2::splat(8.), || {
                                                RoundRect::new(999.).color(t.primary).show();
                                            });
                                        });
                                    }
                                });
                        });
                        label((*option).to_owned());
                    });
                },
            );
        }
    });
}
/// A simple horizontal group of actions using normal component composition.
#[track_caller]
pub fn button_group(children: impl FnOnce()) {
    row(0., children)
}
/// Draw a loading indicator at a deterministic phase (radians).
#[track_caller]
pub fn spinner(size: f32, phase: f32) {
    let t = theme();
    crate::size(Vec2::splat(size), || {
        yakui::widgets::Canvas::new(move |ctx| {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            crate::icons::circle(
                ctx,
                rect.center(),
                size * 0.4,
                size / 12.,
                t.foreground,
                phase,
                phase + 5.1,
            );
        })
        .show();
    });
}
