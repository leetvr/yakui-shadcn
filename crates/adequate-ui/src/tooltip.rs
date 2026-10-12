//! Informational hover/focus popups; triggers and content remain ordinary Yakui nodes.
use crate::{Color, ControlResponse, PopoverAlign, PopoverOptions, Side, Vec2};
use std::{cell::Cell, rc::Rc};
use yakui::{
    WidgetId,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::KeyCode,
    widget::{EventContext, PaintContext, Widget},
    widgets::{Pad, Scope},
};

/// Caller-owned visibility and Escape suppression. Keep one state per trigger.
#[derive(Debug, Default)]
pub struct TooltipState {
    trigger: ControlResponse,
    dismissed: bool,
    open: Rc<Cell<bool>>,
    escape: Rc<Cell<bool>>,
}
impl TooltipState {
    pub fn is_open(&self) -> bool {
        self.open.get() && !self.escape.get()
    }
    fn observe(&mut self, trigger: ControlResponse) {
        self.trigger = trigger;
        if !trigger.hovering && !trigger.focused {
            self.dismissed = false;
        }
        if trigger.clicked {
            self.dismissed = true;
        }
        self.refresh();
    }
    fn refresh(&mut self) {
        if self.escape.replace(false) {
            self.dismissed = true;
        }
        self.open
            .set((self.trigger.hovering || self.trigger.focused) && !self.dismissed);
    }
}
#[derive(Debug)]
pub(crate) struct TooltipTriggerContext {
    open: Rc<Cell<bool>>,
    escape: Rc<Cell<bool>>,
}
impl TooltipTriggerContext {
    pub(crate) fn escape(&self, event: &WidgetEvent) -> bool {
        if self.open.get()
            && matches!(
                event,
                WidgetEvent::KeyChanged {
                    key: KeyCode::Escape,
                    down: true,
                    ..
                }
            )
        {
            self.escape.set(true);
            true
        } else {
            false
        }
    }
}
/// Wrap a normal Button/control so Escape can dismiss without changing focus.
#[track_caller]
pub fn tooltip_trigger(
    state: &mut TooltipState,
    children: impl FnOnce() -> ControlResponse,
) -> ControlResponse {
    let mut trigger = ControlResponse::default();
    Scope::new(TooltipTriggerContext {
        open: state.open.clone(),
        escape: state.escape.clone(),
    })
    .show(|| trigger = children());
    state.observe(trigger);
    trigger
}
#[derive(Debug, Default)]
struct HoverTarget {
    hovering: bool,
}
impl Widget for HoverTarget {
    type Props<'a> = ();
    type Response = ControlResponse;
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, _: ()) -> ControlResponse {
        ControlResponse {
            hovering: self.hovering,
            ..Default::default()
        }
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }
    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => self.hovering = true,
            WidgetEvent::MouseLeave => self.hovering = false,
            _ => {}
        }
        // A hover area must not intercept its child's activation or editor focus.
        EventResponse::Bubble
    }
}
/// Hover-only wrapper for disabled controls or informational icons. No extra tab stop.
#[track_caller]
pub fn tooltip_trigger_area(state: &mut TooltipState, children: impl FnOnce()) -> ControlResponse {
    let result = yakui::util::widget_children::<HoverTarget, _>(children, ());
    let id = result.id;
    let trigger = ControlResponse {
        id: Some(id),
        ..result.into_inner()
    };
    state.observe(trigger);
    trigger
}
/// A compact inverted surface. Declare content at the root after its trigger.
/// Opening is immediate, matching the pinned provider's default delay of zero.
/// Use a popover for interactive content.
///
/// ```no_run
/// use yakui_adequate_ui as ui;
/// let mut tooltip = ui::TooltipState::default();
/// ui::tooltip_trigger(&mut tooltip, || ui::button("Save"));
/// // At the application root:
/// ui::Tooltip::new().side(ui::Side::Top).show(&mut tooltip, "Save changes");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Tooltip {
    options: PopoverOptions,
}
impl Default for Tooltip {
    fn default() -> Self {
        Self {
            options: PopoverOptions {
                width: 320.,
                side: Side::Top,
                align: PopoverAlign::Center,
                ..Default::default()
            },
        }
    }
}
impl Tooltip {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn side(mut self, side: Side) -> Self {
        self.options.side = side;
        self
    }
    pub fn align(mut self, align: PopoverAlign) -> Self {
        self.options.align = align;
        self
    }
    pub fn side_offset(mut self, offset: f32) -> Self {
        self.options.side_offset = offset;
        self
    }
    pub fn max_width(mut self, width: f32) -> Self {
        self.options.width = width;
        self
    }
    #[track_caller]
    pub fn show(self, state: &mut TooltipState, text: impl Into<String>) -> bool {
        self.show_children(state, || crate::text(text))
    }
    /// Compose text/icons/Kbd. Typography and foreground inherit the tooltip theme.
    #[track_caller]
    pub fn show_children(self, state: &mut TooltipState, children: impl FnOnce()) -> bool {
        state.refresh();
        let open = state.is_open();
        crate::animation::presence(
            open,
            crate::animation::Motion::new(
                std::time::Duration::from_millis(100),
                crate::animation::Easing::EaseOut,
            ),
            |sample| {
                if !sample.present {
                    return;
                }
                let t = crate::theme();
                let mut inherited = (*t).clone();
                inherited.foreground = t.background;
                inherited.muted_foreground = t.background;
                inherited.font_size = 12.;
                inherited.line_height = 16.;
                inherited.muted = crate::layout::composite(
                    t.background.with_alpha(if t.dark { 0.1 } else { 0.2 }),
                    t.foreground,
                );
                crate::overlay::tooltip_popup(state.trigger, self.options, || {
                    crate::animation::motion(
                        crate::animation::MotionStyle {
                            opacity: sample.progress,
                            scale: crate::Vec2::splat(0.95 + 0.05 * sample.progress),
                            ..Default::default()
                        },
                        || {
                            crate::with_theme(inherited, || {
                                Scope::new(TooltipContent).show(|| {
                                    yakui::util::widget_children::<Arrow, _>(
                                        || {
                                            crate::surface(
                                                t.foreground,
                                                None,
                                                t.radius * 0.6,
                                                0.,
                                                || {
                                                    yakui::pad(Pad::balanced(12., 6.), || {
                                                        crate::row(6., children)
                                                    });
                                                },
                                            );
                                        },
                                        (state.trigger.id, t.foreground),
                                    );
                                });
                            })
                        },
                    );
                });
            },
        );
        open
    }
}
#[derive(Debug)]
pub(crate) struct TooltipContent;
pub(crate) fn in_tooltip() -> bool {
    yakui::util::read_scope::<TooltipContent>().is_some()
}
#[derive(Debug)]
struct Arrow {
    anchor: Option<WidgetId>,
    color: Color,
}
impl Widget for Arrow {
    type Props<'a> = (Option<WidgetId>, Color);
    type Response = ();
    fn new() -> Self {
        Self {
            anchor: None,
            color: Color::CLEAR,
        }
    }
    fn update(&mut self, (anchor, color): Self::Props<'_>) {
        self.anchor = anchor;
        self.color = color;
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        if let Some(anchor) = self
            .anchor
            .and_then(|id| ctx.layout.get(id))
            .map(|node| node.rect)
        {
            let middle = anchor.center();
            // Determine the actual side after collision flipping, then point at the
            // trigger. This uses final layout positions, not the preferred side.
            let points = if rect.max().y <= anchor.pos().y {
                let x = middle.x.clamp(rect.pos().x + 8., rect.max().x - 8.);
                [
                    Vec2::new(x - 5., rect.max().y - 1.),
                    Vec2::new(x + 5., rect.max().y - 1.),
                    Vec2::new(x, rect.max().y + 5.),
                ]
            } else if rect.pos().y >= anchor.max().y {
                let x = middle.x.clamp(rect.pos().x + 8., rect.max().x - 8.);
                [
                    Vec2::new(x - 5., rect.pos().y + 1.),
                    Vec2::new(x + 5., rect.pos().y + 1.),
                    Vec2::new(x, rect.pos().y - 5.),
                ]
            } else {
                let y = middle.y.clamp(rect.pos().y + 8., rect.max().y - 8.);
                let (edge, direction) = if rect.max().x <= anchor.pos().x {
                    (rect.max().x, 1.)
                } else {
                    (rect.pos().x, -1.)
                };
                [
                    Vec2::new(edge - direction, y - 5.),
                    Vec2::new(edge - direction, y + 5.),
                    Vec2::new(edge + direction * 5., y),
                ]
            };
            let vertices = points
                .map(|point| yakui::paint::Vertex::new(point, Vec2::ZERO, self.color.to_linear()));
            ctx.paint
                .add_mesh(yakui::paint::PaintMesh::new(vertices, [0, 1, 2]));
        }
        self.default_paint(ctx);
    }
}
/// Short hover/focus helper. Use tooltip_trigger + TooltipState for Escape dismissal.
#[track_caller]
pub fn tooltip(trigger: ControlResponse, text: impl Into<String>) {
    crate::layout::with_state(TooltipState::default, |retained| {
        let mut state = retained.borrow_mut();
        state.observe(trigger);
        Tooltip::new().show(&mut state, text);
    });
}
