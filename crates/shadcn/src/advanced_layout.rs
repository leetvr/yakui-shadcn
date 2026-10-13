//! Native aspect ratios, split panels and scoped reading direction.
use crate::Orientation;
use crate::interaction::{Control, ControlWidget};
use yakui::{
    Color, Constraints, Vec2, context,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::{KeyCode, MouseButton},
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
};
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ReadingDirection {
    #[default]
    Ltr,
    Rtl,
}
/// Scope reading direction. Native rows mirror their child positions in RTL.
#[track_caller]
pub fn direction(direction: ReadingDirection, children: impl FnOnce()) {
    yakui::widgets::Scope::new(direction).show(children);
}
pub(crate) fn rtl() -> bool {
    yakui::util::read_scope::<ReadingDirection>().is_some_and(|d| *d == ReadingDirection::Rtl)
}
#[derive(Debug)]
pub(crate) struct RowWidget {
    inner: yakui::widgets::ListWidget,
    rtl: bool,
}
impl Widget for RowWidget {
    type Props<'a> = yakui::widgets::List;
    type Response = ();
    fn new() -> Self {
        Self {
            inner: yakui::widgets::ListWidget::new(),
            rtl: false,
        }
    }
    fn update(&mut self, props: Self::Props<'_>) {
        self.rtl = rtl();
        self.inner.update(props);
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let size = self.inner.layout(
            LayoutContext {
                dom: ctx.dom,
                input: ctx.input,
                layout: ctx.layout,
                paint: ctx.paint,
            },
            constraints,
        );
        if self.rtl {
            for &child in &ctx.dom.get_current().children {
                if ctx.dom.get(child).unwrap().widget.flow() != yakui::Flow::Inline {
                    continue;
                }
                let rect = ctx.layout.get(child).unwrap().rect;
                ctx.layout
                    .set_pos(child, Vec2::new(size.x - rect.max().x, rect.pos().y));
            }
        }
        size
    }
}
#[derive(Debug, Clone, Copy)]
struct AspectProps {
    ratio: f32,
    width: f32,
}
#[derive(Debug)]
struct Aspect {
    props: AspectProps,
}
impl Widget for Aspect {
    type Props<'a> = AspectProps;
    type Response = ();
    fn new() -> Self {
        Self {
            props: AspectProps {
                ratio: 1.,
                width: 320.,
            },
        }
    }
    fn update(&mut self, props: AspectProps) {
        self.props = props;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let ratio = if self.props.ratio.is_finite() && self.props.ratio > 0. {
            self.props.ratio
        } else {
            1.
        };
        let width = self
            .props
            .width
            .min(constraints.max.x)
            .min(constraints.max.y * ratio)
            .max(0.);
        let size = constraints.constrain(Vec2::new(width, width / ratio));
        for &child in &ctx.dom.get_current().children {
            ctx.calculate_layout(child, Constraints::tight(size));
        }
        size
    }
}
/// Keep children in an exact aspect ratio within the available parent constraints.
#[track_caller]
pub fn aspect_ratio(ratio: f32, width: f32, children: impl FnOnce()) {
    yakui::util::widget_children::<Aspect, _>(children, AspectProps { ratio, width });
}
#[derive(Debug, Clone, Copy)]
struct SplitProps {
    size: Vec2,
    orientation: Orientation,
    ratio: f32,
}
#[derive(Debug)]
struct Split {
    props: SplitProps,
}
impl Widget for Split {
    type Props<'a> = SplitProps;
    type Response = ();
    fn new() -> Self {
        Self {
            props: SplitProps {
                size: Vec2::new(480., 240.),
                orientation: Orientation::Horizontal,
                ratio: 0.5,
            },
        }
    }
    fn update(&mut self, props: SplitProps) {
        self.props = props;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let size = input.constrain(self.props.size);
        let horizontal = self.props.orientation == Orientation::Horizontal;
        let total = if horizontal { size.x } else { size.y };
        let available = (total - 6.).max(0.);
        let first = available * self.props.ratio;
        let sizes = [first, 6., available - first];
        let mut offset = 0.;
        for (&child, extent) in ctx.dom.get_current().children.iter().zip(sizes) {
            let constraints = Constraints::tight(if horizontal {
                Vec2::new(extent, size.y)
            } else {
                Vec2::new(size.x, extent)
            });
            ctx.calculate_layout(child, constraints);
            ctx.layout.set_pos(
                child,
                if horizontal {
                    Vec2::new(offset, 0.)
                } else {
                    Vec2::new(0., offset)
                },
            );
            offset += extent;
        }
        size
    }
}
#[derive(Debug, Clone, Copy)]
struct HandleProps {
    orientation: Orientation,
    extent: f32,
    ratio: f32,
    min: f32,
    max: f32,
}
#[derive(Debug)]
struct Handle {
    props: HandleProps,
    control: ControlWidget,
    start: Option<(f32, f32)>,
    pending: Option<f32>,
    color: Color,
}
impl Widget for Handle {
    type Props<'a> = HandleProps;
    type Response = Option<f32>;
    fn new() -> Self {
        Self {
            props: HandleProps {
                orientation: Orientation::Horizontal,
                extent: 480.,
                ratio: 0.5,
                min: 0.1,
                max: 0.9,
            },
            control: ControlWidget::new(),
            start: None,
            pending: None,
            color: Color::GRAY,
        }
    }
    fn update(&mut self, props: HandleProps) -> Self::Response {
        self.props = props;
        let t = crate::theme();
        self.color = crate::layout::resolve_color(t.border);
        self.control.update(Control {
            disabled: false,
            radius: 0.,
            ring: t.ring,
            focus_size: None,
        });
        self.pending.take()
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        self.control.paint(PaintContext {
            dom: ctx.dom,
            layout: ctx.layout,
            paint: ctx.paint,
        });
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let (a, b) = if self.props.orientation == Orientation::Horizontal {
            (
                Vec2::new(rect.center().x, rect.pos().y),
                Vec2::new(rect.center().x, rect.max().y),
            )
        } else {
            (
                Vec2::new(rect.pos().x, rect.center().y),
                Vec2::new(rect.max().x, rect.center().y),
            )
        };
        crate::icons::line(&mut ctx, a, b, 1., self.color);
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL | EventInterest::FOCUSED_KEYBOARD
    }
    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::SEQUENTIAL | FocusPolicy::POINTER
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        let axis = |p: Vec2| {
            if self.props.orientation == Orientation::Horizontal {
                p.x
            } else {
                p.y
            }
        };
        match event {
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                position,
                ..
            } => {
                self.start = Some((axis(*position), self.props.ratio));
                crate::interaction::request_pointer_drag(ctx.dom);
                ctx.input.set_focus(Some(ctx.dom.current()), false);
                return EventResponse::Sink;
            }
            WidgetEvent::MouseMoved(Some(position)) => {
                if let Some((start, ratio)) = self.start {
                    self.pending = Some(
                        (ratio + (axis(*position) - start) / self.props.extent.max(1.))
                            .clamp(self.props.min, self.props.max),
                    );
                    return EventResponse::Sink;
                }
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: false,
                ..
            } => {
                if self.start.take().is_some() {
                    return EventResponse::Sink;
                }
            }
            WidgetEvent::KeyChanged {
                key, down: true, ..
            } => {
                let value = match key {
                    KeyCode::ArrowLeft | KeyCode::ArrowUp => Some(self.props.ratio - 0.02),
                    KeyCode::ArrowRight | KeyCode::ArrowDown => Some(self.props.ratio + 0.02),
                    KeyCode::Home => Some(self.props.min),
                    KeyCode::End => Some(self.props.max),
                    _ => None,
                };
                if let Some(value) = value {
                    self.pending = Some(value.clamp(self.props.min, self.props.max));
                    return EventResponse::Sink;
                }
            }
            _ => {}
        }
        self.control.event(ctx, event)
    }
}
/// Two resizable panels. Nest calls for more panels or mixed orientations.
#[track_caller]
pub fn resizable(
    size: Vec2,
    orientation: Orientation,
    ratio: &mut f32,
    first: impl FnOnce(),
    second: impl FnOnce(),
) {
    if !ratio.is_finite() {
        *ratio = 0.5;
    }
    *ratio = ratio.clamp(0.1, 0.9);
    let dom = context::dom();
    let response = dom.begin_widget::<Split>(SplitProps {
        size,
        orientation,
        ratio: *ratio,
    });
    yakui::widgets::Stack::new().show(first);
    if let Some(next) = yakui::util::widget::<Handle>(HandleProps {
        orientation,
        extent: if orientation == Orientation::Horizontal {
            size.x
        } else {
            size.y
        },
        ratio: *ratio,
        min: 0.1,
        max: 0.9,
    })
    .into_inner()
    {
        *ratio = next;
    }
    yakui::widgets::Stack::new().show(second);
    dom.end_widget::<Split>(response.id);
}
