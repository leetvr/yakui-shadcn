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
    let mut style = t.text_style(t.font_size, t.foreground, 500);
    style.line_height_override = Some(t.font_size);
    RenderText::with_style(text, style).show();
}
/// Typography used by the heading of a group of fields.
#[derive(Debug, Clone, Copy, Default)]
pub enum FieldLegendVariant {
    #[default]
    Legend,
    Label,
}
/// Heading for a field set. Put it before the group's description and controls.
#[track_caller]
pub fn field_legend(text: impl Into<String>, variant: FieldLegendVariant) {
    let t = theme();
    let (size, height) = match variant {
        FieldLegendVariant::Legend => (16., 24.),
        FieldLegendVariant::Label => (14., 20.),
    };
    let mut style = t.text_style(size, t.foreground, 500);
    style.line_height_override = Some(height);
    RenderText::with_style(text, style).show();
}
/// Helper text using the FieldDescription's normal 1.5 line height.
#[track_caller]
pub fn field_description(text: impl Into<String>) {
    let t = theme();
    let mut style = t.text_style(14., t.muted_foreground, 400);
    style.line_height_override = Some(21.);
    RenderText::with_style(text, style).show();
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
        self.show_content(None::<fn(Color)>)
    }
    /// Compose arbitrary native content, such as an Avatar, inside this button.
    #[track_caller]
    pub fn show_children(self, children: impl FnOnce()) -> ControlResponse {
        self.show_content(Some(move |foreground| {
            let mut inherited = (*theme()).clone();
            inherited.foreground = foreground;
            crate::with_theme(inherited, children);
        }))
    }
    #[track_caller]
    fn show_content(self, custom: Option<impl FnOnce(Color)>) -> ControlResponse {
        let custom_content = custom.is_some();
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
                focus_size: None,
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
                    crate::button_group::button_surface(
                        radius,
                        fill,
                        border.map(|c| Border::new(c, t.border_width)),
                        || {
                            yakui::pad(
                                Pad::balanced(
                                    if self.size.icon() { 0. } else { horizontal },
                                    if custom_content {
                                        0.
                                    } else {
                                        (height - line_height) / 2.
                                    },
                                ),
                                || {
                                    let is_icon = self.size.icon();
                                    let content = || {
                                        if let Some(children) = custom {
                                            children(fg);
                                            return;
                                        }
                                        if let Some(icon) = self.icon {
                                            icon_colored(
                                                icon,
                                                if height == 24. { 12. } else { 16. },
                                                fg,
                                            );
                                        }
                                        if !self.size.icon() && !self.text.is_empty() {
                                            let mut style = t.text_style(fs, fg, 500);
                                            style.line_height_override = Some(line_height);
                                            yakui::util::widget_children::<Underline, _>(
                                                || {
                                                    RenderText::with_style(self.text, style).show();
                                                },
                                                (
                                                    matches!(self.variant, Variant::Link) && active,
                                                    fg,
                                                ),
                                            );
                                        }
                                    };
                                    if is_icon {
                                        yakui::center(content);
                                    } else {
                                        crate::min_size(Vec2::ZERO, || row(6., content));
                                    }
                                },
                            );
                        },
                    );
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
    let fill = if fill.a == 0 && border.is_some() {
        crate::layout::surface_background()
    } else {
        crate::layout::resolve_color(fill)
    };
    RoundRect::new(999.)
        .color(fill)
        .border(border.map(|c| Border::new(c, 1.)))
        .show_children(|| {
            yakui::pad(Pad::balanced(9., 2.), || {
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
#[derive(Debug, Clone)]
struct CardSpacing {
    padding: f32,
    footer: std::rc::Rc<std::cell::Cell<bool>>,
}
#[derive(Debug)]
struct CardPadding {
    spacing: CardSpacing,
}
impl yakui::widget::Widget for CardPadding {
    type Props<'a> = CardSpacing;
    type Response = ();
    fn new() -> Self {
        Self {
            spacing: CardSpacing {
                padding: 16.,
                footer: Default::default(),
            },
        }
    }
    fn update(&mut self, spacing: CardSpacing) {
        self.spacing = spacing;
    }
    fn layout(&self, ctx: yakui::widget::LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        // Footer declaration occurs after header/content. Defer the bottom padding
        // decision until layout, when all slots have registered with this card.
        let mut pad = yakui::widgets::PadWidget::new();
        pad.update(
            Pad::ZERO
                .top(self.spacing.padding)
                .bottom(if self.spacing.footer.get() {
                    0.
                } else {
                    self.spacing.padding
                }),
        );
        pad.layout(ctx, constraints)
    }
}
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
            let spacing = CardSpacing {
                padding: spacing,
                footer: Default::default(),
            };
            yakui::widgets::Scope::new(spacing.clone()).show(|| {
                yakui::util::widget_children::<CardPadding, _>(
                    || column(spacing.padding, children),
                    spacing.clone(),
                );
            });
        },
    );
}
fn card_spacing() -> f32 {
    yakui::util::read_scope::<CardSpacing>().map_or(16., |s| s.padding)
}
#[track_caller]
pub fn card_header(children: impl FnOnce()) {
    yakui::pad(Pad::horizontal(card_spacing()), || column(4., children));
}
#[derive(Debug)]
struct CardHeaderSlots;
impl yakui::widget::Widget for CardHeaderSlots {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn layout(&self, mut ctx: yakui::widget::LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let children = ctx.dom.get_current().children.clone();
        let action = ctx.calculate_layout(children[1], Constraints::loose(constraints.max));
        let available = Vec2::new(
            (constraints.max.x - action.x - 12.).max(0.),
            constraints.max.y,
        );
        let content = ctx.calculate_layout(children[0], Constraints::loose(available));
        let size = constraints.constrain(Vec2::new(
            content.x + action.x + 12.,
            content.y.max(action.y),
        ));
        ctx.layout
            .set_pos(children[1], Vec2::new((size.x - action.x).max(0.), 0.));
        size
    }
}
/// Card header with a top-aligned trailing action. Content receives the remaining
/// width, so description wrapping cannot push the action outside the card.
#[track_caller]
pub fn card_header_with_action(content: impl FnOnce(), action: impl FnOnce()) {
    yakui::pad(Pad::horizontal(card_spacing()), || {
        yakui::util::widget_children::<CardHeaderSlots, _>(
            || {
                column(4., content);
                column(0., action);
            },
            (),
        );
    });
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
    if let Some(spacing) = yakui::util::read_scope::<CardSpacing>() {
        spacing.footer.set(true);
    }
    let fill = crate::layout::resolve_color(t.muted.with_alpha(0.5));
    RoundRect::new(yakui::BorderRadius::bottom(t.radius * 1.4))
        .color(fill)
        .show_children(|| {
            column(0., || {
                crate::fill_width(|| {
                    crate::size(Vec2::new(0., t.border_width), || {
                        yakui::widgets::Canvas::new(move |ctx| {
                            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
                            let mut rule = yakui::paint::PaintRect::new(rect);
                            rule.color = t.border;
                            rule.add(ctx.paint);
                        })
                        .show();
                    })
                });
                yakui::pad(Pad::all(card_spacing()), children);
            });
        });
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
    skeleton_at(size, theme().radius * 0.8, 0.)
}
/// Loading placeholder with an application-supplied animation phase in radians.
/// A fixed phase of zero produces a reproducible visual reference.
#[track_caller]
pub fn skeleton_at(size: Vec2, radius: f32, phase: f32) {
    let t = theme();
    let opacity = 0.75 + phase.cos() * 0.25;
    surface(t.muted.with_alpha(opacity), None, radius, 0., || {
        crate::size(size, || {});
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
    crate::size(Vec2::new(width, 4.), || {
        RoundRect::new(999.).color(t.muted).show_children(|| {
            yakui::align(yakui::Alignment::TOP_LEFT, || {
                crate::size(Vec2::new(width * value, 4.), || {
                    RoundRect::new(999.).color(t.primary).show();
                })
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
    Input::new(value).placeholder(placeholder).show().changed
}
/// Native multiline selection, clipboard and cursor behaviour.
#[track_caller]
pub fn textarea(value: &mut String, placeholder: &str) -> bool {
    Input::new(value)
        .placeholder(placeholder)
        .multiline(true)
        .show()
        .changed
}
/// Configurable native editor. Application state is borrowed only while building the frame.
#[derive(Debug)]
pub struct Input<'a> {
    value: &'a mut String,
    placeholder: String,
    disabled: bool,
    read_only: bool,
    invalid: bool,
    multiline: bool,
    password: bool,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct InputResponse {
    pub changed: bool,
    pub submitted: bool,
    pub lost_focus: bool,
    pub id: Option<yakui::WidgetId>,
}
impl<'a> Input<'a> {
    pub fn new(value: &'a mut String) -> Self {
        Self {
            value,
            placeholder: String::new(),
            disabled: false,
            read_only: false,
            invalid: false,
            multiline: false,
            password: false,
        }
    }
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }
    /// Mask graphemes, selection and caret. Password controls are single line.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        if password {
            self.multiline = false;
        }
        self
    }
    #[track_caller]
    pub fn show(self) -> InputResponse {
        let t = theme();
        let mut response = InputResponse::default();
        let fill = if self.disabled {
            t.input.with_alpha(if t.dark { 0.8 } else { 0.5 })
        } else if t.dark {
            t.input.with_alpha(0.045)
        } else {
            Color::CLEAR
        };
        crate::button_group::input_surface(
            fill,
            if self.invalid { t.destructive } else { t.input },
            t.radius,
            || {
                let foreground = if self.disabled {
                    crate::layout::resolve_color(t.foreground.with_alpha(0.5))
                } else {
                    t.foreground
                };
                let textbox = yakui::widgets::TextBox::new(self.value.clone())
                    .style(t.text_style(t.font_size, foreground, 400))
                    .padding(Pad::balanced(10., 6.))
                    .fill(None)
                    .radius(t.radius)
                    .placeholder(self.placeholder)
                    .cursor_color(t.foreground)
                    .selection_halo_color(Color::CLEAR)
                    .selected_bg_color(t.accent)
                    .multiline(self.multiline)
                    .inline_edit(!self.multiline);
                ConstrainedBox::new(Constraints {
                    min: Vec2::new(0., if self.multiline { 64. } else { 32. }),
                    max: Vec2::new(
                        f32::INFINITY,
                        if self.multiline { f32::INFINITY } else { 32. },
                    ),
                })
                .show(|| {
                    let props = crate::editor::EditorProps {
                        textbox,
                        disabled: self.disabled,
                        read_only: self.read_only,
                        invalid: self.invalid,
                    };
                    let res = if self.password {
                        yakui::util::widget::<crate::password::Password>(props)
                    } else {
                        yakui::util::widget::<crate::editor::Editor>(props)
                    };
                    response.id = Some(res.id);
                    let res = res.into_inner();
                    response.submitted = res.activated;
                    response.lost_focus = res.lost_focus;
                    if let Some(next) = res.text {
                        response.changed = *self.value != next;
                        *self.value = next;
                    }
                });
            },
        );
        response
    }
}
/// Checkbox and label share one hit target and keyboard focus.
#[track_caller]
pub fn checkbox(label_text: impl Into<String>, value: &mut bool) -> ControlResponse {
    crate::Checkbox::new().label(label_text).show(value)
}
/// A labelled checkbox that skips interaction and focus when disabled.
#[track_caller]
pub fn checkbox_disabled(
    label_text: impl Into<String>,
    value: &mut bool,
    disabled: bool,
) -> ControlResponse {
    crate::Checkbox::new()
        .label(label_text)
        .disabled(disabled)
        .show(value)
}
/// A two-state compact control, with explicit disabled state.
#[track_caller]
pub fn switch(label_text: impl Into<String>, value: &mut bool, disabled: bool) -> ControlResponse {
    crate::Switch::new()
        .label(label_text)
        .disabled(disabled)
        .show(value)
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
/// Single-selection radio group. Tab enters at the selection; arrows/Home/End
/// move focus and select, wrapping at either end of the group.
#[track_caller]
pub fn radio_group(options: &[&str], selected: &mut usize) {
    let t = theme();
    column(8., || {
        crate::navigation::choice_group(options.len(), selected, |i, selected, _response| {
            row(12., || {
                crate::size(Vec2::splat(16.), || {
                    RoundRect::new(999.)
                        .color(if *selected == i {
                            t.primary
                        } else {
                            crate::layout::surface_background()
                        })
                        .border(Some(Border::new(
                            if *selected == i {
                                t.primary
                            } else {
                                crate::layout::resolve_color(t.input)
                            },
                            1.,
                        )))
                        .show_children(|| {
                            if *selected == i {
                                yakui::pad(Pad::all(4.), || {
                                    crate::size(Vec2::splat(8.), || {
                                        RoundRect::new(999.).color(t.primary_foreground).show();
                                    });
                                });
                            }
                        });
                });
                let mut style = t.text_style(14., t.foreground, 500);
                style.line_height_override = Some(14.);
                RenderText::with_style(options[i].to_owned(), style).show();
            });
        });
    });
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

/// Caller-owned toolbar selections. Single mode permits at most one pressed action.
#[derive(Debug, Default, Clone)]
pub struct ToggleGroupState {
    pub selected: std::collections::BTreeSet<usize>,
    pub single: bool,
}
#[track_caller]
pub fn toggle_group(options: &[&str], state: &mut ToggleGroupState, outline: bool) {
    row(0., || {
        for (i, option) in options.iter().enumerate() {
            let pressed = state.selected.contains(&i);
            if Button::new(*option)
                .variant(if pressed {
                    Variant::Secondary
                } else if outline {
                    Variant::Outline
                } else {
                    Variant::Ghost
                })
                .show()
                .clicked
            {
                if pressed {
                    state.selected.remove(&i);
                } else {
                    if state.single {
                        state.selected.clear();
                    }
                    state.selected.insert(i);
                }
            }
        }
    });
}

#[derive(Debug)]
struct Underline {
    enabled: bool,
    color: Color,
}
impl yakui::widget::Widget for Underline {
    type Props<'a> = (bool, Color);
    type Response = ();
    fn new() -> Self {
        Self {
            enabled: false,
            color: Color::BLACK,
        }
    }
    fn update(&mut self, props: Self::Props<'_>) {
        self.enabled = props.0;
        self.color = props.1;
    }
    fn paint(&self, mut ctx: yakui::widget::PaintContext<'_>) {
        for &child in &ctx.dom.get_current().children {
            ctx.paint(child);
        }
        if self.enabled {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            let mut line = yakui::paint::PaintRect::new(yakui::Rect::from_pos_size(
                Vec2::new(rect.pos().x, rect.max().y),
                Vec2::new(rect.size().x, 1.),
            ));
            line.color = self.color;
            line.add(ctx.paint);
        }
    }
}
/// Inline link action. The caller owns routing; hover paints the upstream underline.
#[track_caller]
pub fn link(text: impl Into<String>) -> ControlResponse {
    let t = theme();
    control(
        Control {
            disabled: false,
            radius: 0.,
            ring: t.ring,
            focus_size: None,
        },
        |response| {
            yakui::util::widget_children::<Underline, _>(
                || {
                    RenderText::with_style(text, t.text_style(14., t.primary, 400)).show();
                },
                (response.hovering, t.primary),
            );
        },
    )
}
