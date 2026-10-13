//! Presentational chat parts. Transport, upload work and business actions stay in the app.
use crate::{Button, ButtonSize, Color, Variant, Vec2, column, row, surface, theme};
#[derive(Debug, Default, Clone, Copy)]
pub enum BubbleVariant {
    #[default]
    Default,
    Secondary,
    Muted,
    Tinted,
    Outline,
    Ghost,
    Destructive,
}
/// Chat bubble whose children may contain any existing ported shadcn/ui component.
#[track_caller]
pub fn bubble(variant: BubbleVariant, children: impl FnOnce()) {
    let t = theme();
    let (fill, foreground, border) = match variant {
        BubbleVariant::Default => (t.primary, t.primary_foreground, None),
        BubbleVariant::Secondary => (t.secondary, t.secondary_foreground, None),
        BubbleVariant::Muted => (t.muted, t.foreground, None),
        BubbleVariant::Tinted => (
            crate::layout::composite(
                t.primary.with_alpha(if t.dark { 0.15 } else { 0.08 }),
                t.background,
            ),
            t.foreground,
            None,
        ),
        BubbleVariant::Outline => (t.background, t.foreground, Some(t.border)),
        BubbleVariant::Ghost => (Color::CLEAR, t.foreground, None),
        BubbleVariant::Destructive => (
            t.destructive.with_alpha(if t.dark { 0.2 } else { 0.1 }),
            t.destructive,
            None,
        ),
    };
    surface(
        fill,
        border,
        if matches!(variant, BubbleVariant::Ghost) {
            0.
        } else {
            t.radius * 1.4
        },
        if matches!(variant, BubbleVariant::Ghost) {
            0.
        } else {
            10.
        },
        || {
            let mut inner = (*t).clone();
            inner.foreground = foreground;
            crate::with_theme(inner, children);
        },
    );
}
#[derive(Debug, Default, Clone, Copy)]
pub enum MessageAlign {
    #[default]
    Start,
    End,
}
/// Avatar, header, body and timestamp. End alignment mirrors the message row.
#[track_caller]
pub fn message(
    author: &str,
    initials: &str,
    timestamp: &str,
    align: MessageAlign,
    children: impl FnOnce(),
) {
    crate::direction(
        if matches!(align, MessageAlign::End) {
            crate::ReadingDirection::Rtl
        } else {
            crate::ReadingDirection::Ltr
        },
        || {
            row(8., || {
                crate::avatar(initials, 32.);
                column(8., || {
                    crate::muted(author);
                    children();
                    crate::muted(timestamp);
                });
            })
        },
    );
}
#[derive(Debug, Default, Clone, Copy)]
pub enum MarkerVariant {
    #[default]
    Default,
    Separator,
    Border,
}
#[derive(Debug)]
struct FlexibleRule {
    color: Color,
}
impl yakui::widget::Widget for FlexibleRule {
    type Props<'a> = Color;
    type Response = ();
    fn new() -> Self {
        Self { color: Color::GRAY }
    }
    fn update(&mut self, color: Color) {
        self.color = color;
    }
    fn flex(&self) -> (u32, yakui::FlexFit) {
        (1, yakui::FlexFit::Tight)
    }
    fn layout(&self, _: yakui::widget::LayoutContext<'_>, constraints: yakui::Constraints) -> Vec2 {
        constraints.constrain(Vec2::new(constraints.max.x, 1.))
    }
    fn paint(&self, ctx: yakui::widget::PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        yakui::shapes::RoundedRectangle::new(rect, 0.)
            .color(self.color)
            .add(ctx.paint);
    }
}
/// Date/event marker, optionally dividing the transcript with horizontal rules.
#[track_caller]
pub fn marker(text: impl Into<String>, variant: MarkerVariant, width: f32) {
    let t = theme();
    crate::width(width, || {
        column(8., || {
            row(12., || {
                if matches!(variant, MarkerVariant::Separator) {
                    yakui::util::widget::<FlexibleRule>(crate::layout::resolve_color(t.border));
                }
                crate::muted(text);
                if matches!(variant, MarkerVariant::Separator) {
                    yakui::util::widget::<FlexibleRule>(crate::layout::resolve_color(t.border));
                }
            });
            if matches!(variant, MarkerVariant::Border) {
                crate::separator(width);
            }
        })
    });
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentState {
    Idle,
    Uploading,
    Processing,
    Error,
    #[default]
    Done,
}
#[derive(Debug, Default, Clone, Copy)]
pub enum AttachmentSize {
    #[default]
    Default,
    Sm,
    Xs,
}
#[derive(Debug, Clone)]
pub struct Attachment {
    pub title: String,
    pub description: String,
    pub state: AttachmentState,
    pub size: AttachmentSize,
    pub orientation: crate::Orientation,
    pub progress: f32,
}
impl Attachment {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            state: AttachmentState::Done,
            size: AttachmentSize::Default,
            orientation: crate::Orientation::Horizontal,
            progress: 0.,
        }
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct AttachmentResponse {
    pub opened: bool,
    pub removed: bool,
    pub retry: bool,
}
/// Attachment card: its actions return requests instead of performing file or network I/O.
#[track_caller]
pub fn attachment(attachment: &Attachment) -> AttachmentResponse {
    let t = theme();
    let mut response = AttachmentResponse::default();
    let padding = match attachment.size {
        AttachmentSize::Default => 8.,
        AttachmentSize::Sm => 6.,
        AttachmentSize::Xs => 4.,
    };
    surface(
        t.card,
        Some(if attachment.state == AttachmentState::Error {
            t.destructive
        } else {
            t.border
        }),
        t.radius * 1.4,
        padding,
        || {
            let content = || {
                crate::size(
                    Vec2::splat(match attachment.size {
                        AttachmentSize::Default => 40.,
                        AttachmentSize::Sm => 32.,
                        AttachmentSize::Xs => 28.,
                    }),
                    || {
                        surface(t.muted, None, t.radius, 0., || {
                            yakui::center(|| {
                                if matches!(
                                    attachment.state,
                                    AttachmentState::Uploading | AttachmentState::Processing
                                ) {
                                    crate::spinner(16., 0.);
                                } else {
                                    crate::icon(crate::Icon::Mail, 16.);
                                }
                            });
                        })
                    },
                );
                column(2., || {
                    crate::label(&attachment.title);
                    crate::muted(&attachment.description);
                    if attachment.state == AttachmentState::Uploading {
                        crate::progress(attachment.progress, 120.);
                    }
                    if attachment.state == AttachmentState::Error {
                        crate::text("Upload failed");
                    }
                });
                if Button::new(if attachment.state == AttachmentState::Error {
                    "Retry"
                } else {
                    "Open"
                })
                .size(ButtonSize::Xs)
                .variant(Variant::Ghost)
                .show()
                .clicked
                {
                    if attachment.state == AttachmentState::Error {
                        response.retry = true;
                    } else {
                        response.opened = true;
                    }
                }
                response.removed = Button::new("")
                    .icon(crate::Icon::X)
                    .size(ButtonSize::IconXs)
                    .variant(Variant::Ghost)
                    .show()
                    .clicked;
            };
            if attachment.orientation == crate::Orientation::Vertical {
                column(8., content)
            } else {
                row(8., content)
            }
        },
    );
    response
}
/// An item row surface with default, outlined or muted styling.
#[track_caller]
pub fn item(variant: Variant, children: impl FnOnce()) {
    let t = theme();
    surface(
        if variant == Variant::Secondary {
            t.muted
        } else {
            Color::CLEAR
        },
        if variant == Variant::Outline {
            Some(t.border)
        } else {
            None
        },
        t.radius,
        12.,
        children,
    )
}
/// Form composition. Submit/validation work is expressed by ordinary action responses.
#[track_caller]
pub fn form(children: impl FnOnce()) {
    column(16., children)
}
