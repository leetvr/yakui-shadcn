//! Explicit overlay state and a Yakui layer with modal focus ownership.
//! Render overlays as siblings of the application content (outside scroll areas).
use crate::{ControlResponse, column, surface, theme};
use std::{cell::Cell, rc::Rc};
use yakui::{Alignment, Color, Constraints, Vec2, WidgetId, widgets::Scope};
use yakui_core::{
    navigation::NavDirection,
    widget::{LayoutContext, NavigateContext, Widget},
};

/// Application-owned visibility and return-focus target for a popup.
#[derive(Debug, Default)]
pub struct OverlayState {
    pub open: bool,
    pub trigger: Option<WidgetId>,
    pub anchor_position: Option<Vec2>,
    dismiss: Rc<Cell<bool>>,
}
impl Clone for OverlayState {
    fn clone(&self) -> Self {
        Self {
            open: self.open,
            trigger: self.trigger,
            anchor_position: self.anchor_position,
            dismiss: Rc::new(Cell::new(self.dismiss.get())),
        }
    }
}
impl OverlayState {
    /// Open from the actual trigger response, allowing focus to be restored on dismissal.
    pub fn open_from(&mut self, response: ControlResponse) {
        self.trigger = response.id;
        self.anchor_position = None;
        self.open = true;
        self.dismiss.set(false);
    }
    pub fn close(&mut self) {
        self.open = false;
        self.dismiss.set(false);
    }
    pub fn toggle_from(&mut self, response: ControlResponse) {
        if self.open {
            self.close()
        } else {
            self.open_from(response)
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct DismissContext {
    pub dismiss: Rc<Cell<bool>>,
    pub trigger: Option<WidgetId>,
}
/// Escape is intercepted by controls and editors before their own handling.
pub(crate) fn escape(
    ctx: &yakui_core::widget::EventContext<'_>,
    event: &yakui_core::event::WidgetEvent,
    scope: Option<&DismissContext>,
) -> bool {
    if matches!(
        event,
        yakui_core::event::WidgetEvent::KeyChanged {
            key: yakui_core::input::KeyCode::Escape,
            down: true,
            ..
        }
    ) {
        if let Some(scope) = scope {
            scope.dismiss.set(true);
            ctx.input.set_focus(scope.trigger, false);
            return true;
        }
    }
    false
}
#[derive(Debug, Clone)]
struct FocusProps {
    open: bool,
    modal: bool,
    auto_focus: bool,
    trigger: Option<WidgetId>,
    dismiss: Rc<Cell<bool>>,
}
#[derive(Debug)]
struct FocusScope {
    props: FocusProps,
    was_open: Cell<bool>,
}
impl Widget for FocusScope {
    type Props<'a> = FocusProps;
    type Response = ();
    fn new() -> Self {
        Self {
            props: FocusProps {
                open: false,
                modal: true,
                auto_focus: true,
                trigger: None,
                dismiss: Rc::default(),
            },
            was_open: Cell::new(false),
        }
    }
    fn update(&mut self, props: FocusProps) {
        self.props = props;
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let was_open = self.was_open.replace(self.props.open);
        if !self.props.open {
            if was_open {
                ctx.input.set_focus(self.props.trigger, false);
            }
            return Vec2::ZERO;
        }
        ctx.layout.new_layer(ctx.dom);
        let size = self.default_layout(
            yakui_core::widget::LayoutContext {
                dom: ctx.dom,
                input: ctx.input,
                layout: ctx.layout,
                paint: ctx.paint,
            },
            constraints,
        );
        if !was_open && self.props.auto_focus {
            let nav = NavigateContext {
                dom: ctx.dom,
                input: ctx.input,
                layout: ctx.layout,
            };
            if let Some(focus) = self.default_navigate(nav, NavDirection::Next) {
                ctx.input.set_focus(Some(focus), true);
            }
        }
        size
    }
    fn navigate(&self, ctx: NavigateContext<'_>, dir: NavDirection) -> Option<WidgetId> {
        if !self.props.open {
            return None;
        }
        if let Some(next) = self.default_navigate(ctx, dir) {
            return Some(next);
        }
        if self.props.modal && matches!(dir, NavDirection::Next | NavDirection::Previous) {
            fn collect(ctx: NavigateContext<'_>, id: WidgetId, out: &mut Vec<WidgetId>) {
                let node = ctx.dom.get(id).unwrap();
                if node
                    .widget
                    .focus_policy()
                    .contains(yakui_core::widget::FocusPolicy::SEQUENTIAL)
                {
                    out.push(id);
                }
                for &child in &node.children {
                    collect(ctx, child, out);
                }
            }
            let mut targets = Vec::new();
            collect(ctx, ctx.dom.current(), &mut targets);
            if let Some(id) = if dir == NavDirection::Next {
                targets.first()
            } else {
                targets.last()
            } {
                return Some(*id);
            }
            // A modal with no interactive children still owns focus.
            return Some(ctx.dom.current());
        }
        None
    }
    fn event_interest(&self) -> yakui_core::event::EventInterest {
        if self.props.open {
            yakui_core::event::EventInterest::FOCUSED_KEYBOARD
        } else {
            yakui_core::event::EventInterest::empty()
        }
    }
    fn event(
        &mut self,
        ctx: yakui_core::widget::EventContext<'_>,
        event: &yakui_core::event::WidgetEvent,
    ) -> yakui_core::event::EventResponse {
        if matches!(
            event,
            yakui_core::event::WidgetEvent::KeyChanged {
                key: yakui_core::input::KeyCode::Escape,
                down: true,
                ..
            }
        ) {
            self.props.dismiss.set(true);
            ctx.input.set_focus(self.props.trigger, false);
            return yakui_core::event::EventResponse::Sink;
        }
        yakui_core::event::EventResponse::Bubble
    }
}
#[derive(Debug, Clone, Copy)]
struct BackdropProps {
    fill: Color,
    dismissible: bool,
}
#[derive(Debug)]
struct Backdrop {
    props: BackdropProps,
    dismissed: bool,
}
impl Widget for Backdrop {
    type Props<'a> = BackdropProps;
    type Response = bool;
    fn new() -> Self {
        Self {
            props: BackdropProps {
                fill: Color::CLEAR,
                dismissible: true,
            },
            dismissed: false,
        }
    }
    fn update(&mut self, props: BackdropProps) -> bool {
        self.props = props;
        std::mem::take(&mut self.dismissed)
    }
    fn paint(&self, ctx: yakui_core::widget::PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        yakui::shapes::RoundedRectangle::new(rect, 0.)
            .color(self.props.fill)
            .add(ctx.paint);
    }
    fn event_interest(&self) -> yakui_core::event::EventInterest {
        yakui_core::event::EventInterest::MOUSE_INSIDE
    }
    fn event(
        &mut self,
        _ctx: yakui_core::widget::EventContext<'_>,
        event: &yakui_core::event::WidgetEvent,
    ) -> yakui_core::event::EventResponse {
        if let yakui_core::event::WidgetEvent::MouseButtonChanged {
            button: yakui_core::input::MouseButton::One,
            down: true,
            inside: true,
            ..
        } = event
        {
            self.dismissed = self.props.dismissible;
            return yakui_core::event::EventResponse::Sink;
        }
        if matches!(event, yakui_core::event::WidgetEvent::MouseScroll { .. }) {
            return yakui_core::event::EventResponse::Sink;
        }
        yakui_core::event::EventResponse::Bubble
    }
}
#[derive(Debug, Clone, Copy)]
struct Placement {
    anchor: Alignment,
    edge: Option<Side>,
    width: f32,
}
#[derive(Debug)]
struct PlacementWidget {
    props: Placement,
}
impl Widget for PlacementWidget {
    type Props<'a> = Placement;
    type Response = ();
    fn new() -> Self {
        Self {
            props: Placement {
                anchor: Alignment::CENTER,
                edge: None,
                width: 384.,
            },
        }
    }
    fn update(&mut self, props: Placement) {
        self.props = props;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let viewport = input.max;
        let side = self.props.edge;
        let width = if matches!(side, Some(Side::Top | Side::Bottom)) {
            viewport.x
        } else {
            self.props
                .width
                .min(if side.is_some() {
                    viewport.x
                } else {
                    viewport.x - 32.
                })
                .max(0.)
        };
        let full_height = matches!(side, Some(Side::Left | Side::Right));
        for &child in &ctx.dom.get_current().children {
            let size = ctx.calculate_layout(
                child,
                Constraints {
                    min: Vec2::new(width, if full_height { viewport.y } else { 0. }),
                    max: Vec2::new(
                        width,
                        if side.is_some() {
                            viewport.y
                        } else {
                            (viewport.y - 32.).max(0.)
                        },
                    ),
                },
            );
            ctx.layout
                .set_pos(child, (viewport - size) * self.props.anchor.as_vec2());
        }
        viewport
    }
    fn event_interest(&self) -> yakui_core::event::EventInterest {
        yakui_core::event::EventInterest::empty()
    }
}
#[derive(Debug)]
struct BlockPointer;
impl Widget for BlockPointer {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn event_interest(&self) -> yakui_core::event::EventInterest {
        yakui_core::event::EventInterest::MOUSE_INSIDE
    }
    fn event(
        &mut self,
        _: yakui_core::widget::EventContext<'_>,
        event: &yakui_core::event::WidgetEvent,
    ) -> yakui_core::event::EventResponse {
        if matches!(
            event,
            yakui_core::event::WidgetEvent::MouseButtonChanged { .. }
                | yakui_core::event::WidgetEvent::MouseScroll { .. }
        ) {
            yakui_core::event::EventResponse::Sink
        } else {
            yakui_core::event::EventResponse::Bubble
        }
    }
}
#[track_caller]
fn modal(
    state: &mut OverlayState,
    width: f32,
    anchor: Alignment,
    dismissible: bool,
    edge: Option<Side>,
    children: impl FnOnce(&mut OverlayState),
) {
    if state.dismiss.take() {
        state.open = false;
    }
    let t = theme();
    let props = FocusProps {
        open: state.open,
        modal: true,
        auto_focus: true,
        trigger: state.trigger,
        dismiss: state.dismiss.clone(),
    };
    yakui::util::widget_children::<FocusScope, _>(
        || {
            if !state.open {
                return;
            }
            let dismissed = yakui::util::widget::<Backdrop>(BackdropProps {
                fill: Color::BLACK.with_alpha(0.1),
                dismissible,
            })
            .into_inner();
            if dismissed {
                state.open = false;
                return;
            }
            Scope::new(DismissContext {
                dismiss: state.dismiss.clone(),
                trigger: state.trigger,
            })
            .show(|| {
                yakui::util::widget_children::<PlacementWidget, _>(
                    || {
                        yakui::util::widget_children::<BlockPointer, _>(
                            || {
                                surface(
                                    if edge.is_some() {
                                        t.background
                                    } else {
                                        t.popover
                                    },
                                    Some(t.foreground.with_alpha(0.1)),
                                    if edge.is_some() { 0. } else { t.radius * 1.4 },
                                    16.,
                                    || column(16., || children(state)),
                                )
                            },
                            (),
                        );
                    },
                    Placement {
                        width,
                        anchor,
                        edge,
                    },
                );
            });
        },
        props,
    );
}
/// Modal dialog. Render at the application root after ordinary content.
/// Pass the state into the content closure to close it from an action.
#[track_caller]
pub fn dialog(state: &mut OverlayState, children: impl FnOnce(&mut OverlayState)) {
    modal(state, 384., Alignment::CENTER, true, None, children)
}
/// Confirmation dialog. Backdrop clicks are blocked; Escape and explicit actions dismiss.
#[track_caller]
pub fn alert_dialog(state: &mut OverlayState, children: impl FnOnce(&mut OverlayState)) {
    modal(state, 384., Alignment::CENTER, false, None, children)
}
/// Edge of a sheet or drawer.
#[derive(Debug, Clone, Copy, Default)]
pub enum Side {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}
/// A modal sheet anchored to an edge. Uses the same focus/dismissal contract as Dialog.
#[track_caller]
pub fn sheet(state: &mut OverlayState, side: Side, children: impl FnOnce(&mut OverlayState)) {
    let anchor = match side {
        Side::Left => Alignment::CENTER_LEFT,
        Side::Right => Alignment::CENTER_RIGHT,
        Side::Top => Alignment::TOP_CENTER,
        Side::Bottom => Alignment::BOTTOM_CENTER,
    };
    modal(state, 384., anchor, true, Some(side), children)
}
/// A bottom modal drawer. Pointer dragging is not yet implemented.
#[track_caller]
pub fn drawer(state: &mut OverlayState, children: impl FnOnce(&mut OverlayState)) {
    sheet(state, Side::Bottom, children)
}

#[derive(Debug, Clone, Copy)]
struct PopupPlacement {
    anchor: Option<WidgetId>,
    point: Option<Vec2>,
    side: Side,
    width: f32,
}
#[derive(Debug)]
struct PopupPlacementWidget {
    props: PopupPlacement,
}
impl Widget for PopupPlacementWidget {
    type Props<'a> = PopupPlacement;
    type Response = ();
    fn new() -> Self {
        Self {
            props: PopupPlacement {
                anchor: None,
                point: None,
                side: Side::Bottom,
                width: 240.,
            },
        }
    }
    fn update(&mut self, props: PopupPlacement) {
        self.props = props;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let viewport = input.max;
        let width = self.props.width.min(viewport.x - 16.).max(0.);
        let anchor = self
            .props
            .anchor
            .and_then(|id| ctx.rect_relative_to(id, ctx.dom.root()))
            .unwrap_or_else(|| yakui::Rect::from_pos_size(viewport / 2., Vec2::ZERO));
        let anchor = self
            .props
            .point
            .map_or(anchor, |p| yakui::Rect::from_pos_size(p, Vec2::ZERO));
        for &child in &ctx.dom.get_current().children {
            let size = ctx.calculate_layout(
                child,
                Constraints {
                    min: Vec2::new(width, 0.),
                    max: Vec2::new(width, (viewport.y - 16.).max(0.)),
                },
            );
            let preferred = match self.props.side {
                Side::Right => Vec2::new(anchor.max().x + 4., anchor.pos().y),
                Side::Left => Vec2::new(anchor.pos().x - size.x - 4., anchor.pos().y),
                Side::Top => Vec2::new(anchor.pos().x, anchor.pos().y - size.y - 4.),
                Side::Bottom => Vec2::new(anchor.pos().x, anchor.max().y + 4.),
            };
            let mut pos = preferred;
            if matches!(self.props.side, Side::Right) && pos.x + size.x > viewport.x - 8. {
                pos.x = anchor.pos().x - size.x - 4.;
            }
            if matches!(self.props.side, Side::Bottom) && pos.y + size.y > viewport.y - 8. {
                pos.y = anchor.pos().y - size.y - 4.;
            }
            pos = pos.clamp(
                Vec2::splat(8.),
                (viewport - size - Vec2::splat(8.)).max(Vec2::splat(8.)),
            );
            ctx.layout.set_pos(child, pos);
        }
        viewport
    }
}
#[track_caller]
fn popup(
    state: &mut OverlayState,
    width: f32,
    interactive: bool,
    side: Side,
    children: impl FnOnce(&mut OverlayState),
) {
    if state.dismiss.take() {
        state.open = false;
    }
    let props = FocusProps {
        open: state.open,
        modal: false,
        auto_focus: interactive,
        trigger: state.trigger,
        dismiss: state.dismiss.clone(),
    };
    let placement = PopupPlacement {
        anchor: state.trigger,
        point: state.anchor_position,
        side,
        width,
    };
    let t = theme();
    yakui::util::widget_children::<FocusScope, _>(
        || {
            if !state.open {
                return;
            }
            if interactive
                && yakui::util::widget::<Backdrop>(BackdropProps {
                    fill: Color::CLEAR,
                    dismissible: true,
                })
                .into_inner()
            {
                state.close();
                return;
            }
            Scope::new(DismissContext {
                dismiss: state.dismiss.clone(),
                trigger: state.trigger,
            })
            .show(|| {
                yakui::util::widget_children::<PopupPlacementWidget, _>(
                    || {
                        yakui::util::widget_children::<BlockPointer, _>(
                            || {
                                surface(
                                    t.popover,
                                    Some(t.foreground.with_alpha(0.1)),
                                    t.radius,
                                    8.,
                                    || children(state),
                                )
                            },
                            (),
                        );
                    },
                    placement,
                );
            });
        },
        props,
    );
}
/// An anchored popup with viewport collision avoidance, Escape and outside dismissal.
/// Render at the application root after the trigger; the state retains its WidgetId.
#[track_caller]
pub fn popover(state: &mut OverlayState, width: f32, children: impl FnOnce(&mut OverlayState)) {
    popup(state, width, true, Side::Bottom, children)
}
/// Tooltip content anchored to a trigger. The caller supplies the trigger response.
/// This initial API displays on hover/focus; delayed opening is recorded as a deviation.
#[track_caller]
pub fn tooltip(trigger: ControlResponse, text: impl Into<String>) {
    let mut state = OverlayState {
        open: trigger.hovering || trigger.focused,
        trigger: trigger.id,
        ..OverlayState::default()
    };
    popup(&mut state, 200., false, Side::Bottom, |_| crate::text(text));
}
/// A hover-card surface. Caller controls visibility so content can remain open when hovered.
#[track_caller]
pub fn hover_card(state: &mut OverlayState, width: f32, children: impl FnOnce(&mut OverlayState)) {
    popup(state, width, false, Side::Bottom, children)
}

/// Popup with a preferred side. Placement flips/clamps against the viewport.
#[track_caller]
pub fn popover_on_side(
    state: &mut OverlayState,
    width: f32,
    side: Side,
    children: impl FnOnce(&mut OverlayState),
) {
    popup(state, width, true, side, children)
}
