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
    backdrop: bool,
    trigger: Option<WidgetId>,
    dismiss: Rc<Cell<bool>>,
}
#[derive(Debug)]
struct FocusScope {
    props: FocusProps,
    was_open: Cell<bool>,
    owned_focus: Cell<bool>,
}
impl FocusScope {
    fn contains_focus(&self, ctx: &LayoutContext<'_>) -> bool {
        let mut current = ctx.input.focus();
        while let Some(id) = current {
            if id == ctx.dom.current() {
                return true;
            }
            current = ctx.dom.get(id).and_then(|node| node.parent);
        }
        false
    }
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
                backdrop: true,
                trigger: None,
                dismiss: Rc::default(),
            },
            was_open: Cell::new(false),
            owned_focus: Cell::new(false),
        }
    }
    fn update(&mut self, props: FocusProps) {
        self.props = props;
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let was_open = self.was_open.replace(self.props.open);
        if !self.props.open {
            // Pointer-only tooltips never own focus. Returning it on close would
            // focus their trigger and reopen them. Preserve a new outside target
            // too, while restoring focus when a focused popup child disappears.
            let owned = self.owned_focus.replace(false);
            let moved_outside = ctx
                .input
                .focus()
                .is_some_and(|id| ctx.dom.get(id).is_some() && !self.contains_focus(&ctx));
            // A root menu's backdrop also owns focus in its sibling submenus,
            // which can disappear together when a deep action closes the tree.
            if was_open && (self.props.modal || ((owned || self.props.backdrop) && !moved_outside))
            {
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
        self.owned_focus.set(self.contains_focus(&ctx));
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
        // Yakui dispatches a layer's children in layout insertion order. Put
        // content above its earlier backdrop so controls consume pointer events
        // before the full-window dismissal target, while backdrop paint stays below.
        ctx.layout.new_layer(ctx.dom);
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
        backdrop: true,
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

/// Cross-axis alignment of a popup relative to its trigger.
#[derive(Debug, Default, Clone, Copy)]
pub enum PopoverAlign {
    #[default]
    Start,
    Center,
    End,
}
/// Native popup placement, matching the upstream side/align/offset vocabulary.
#[derive(Debug, Clone, Copy)]
pub struct PopoverOptions {
    pub width: f32,
    pub side: Side,
    pub align: PopoverAlign,
    pub side_offset: f32,
    pub align_offset: f32,
}
impl Default for PopoverOptions {
    fn default() -> Self {
        Self {
            width: 240.,
            side: Side::Bottom,
            align: PopoverAlign::Start,
            side_offset: 4.,
            align_offset: 0.,
        }
    }
}
#[derive(Debug, Clone, Copy)]
struct PopupPlacement {
    anchor: Option<WidgetId>,
    point: Option<Vec2>,
    options: PopoverOptions,
    rtl: bool,
    bounded_to_side: bool,
    minimum_width: f32,
    height_offset: Option<f32>,
    fit_width: bool,
}
#[derive(Debug)]
struct PopupPlacementWidget {
    props: PopupPlacement,
    last_height: Cell<f32>,
}
impl Widget for PopupPlacementWidget {
    type Props<'a> = PopupPlacement;
    type Response = ();
    fn new() -> Self {
        Self {
            last_height: Cell::new(0.),
            props: PopupPlacement {
                anchor: None,
                point: None,
                options: PopoverOptions::default(),
                rtl: false,
                bounded_to_side: false,
                minimum_width: 128.,
                height_offset: None,
                fit_width: false,
            },
        }
    }
    fn update(&mut self, props: PopupPlacement) {
        self.props = props;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        // Yakui dispatches a layer's children in layout insertion order. Put
        // content above its earlier backdrop so controls consume pointer events
        // before the full-window dismissal target, while backdrop paint stays below.
        ctx.layout.new_layer(ctx.dom);
        let viewport = input.max;
        let anchor = self
            .props
            .anchor
            .and_then(|id| ctx.rect_relative_to(id, ctx.dom.root()))
            .unwrap_or_else(|| yakui::Rect::from_pos_size(viewport / 2., Vec2::ZERO));
        let anchor = self
            .props
            .point
            .map_or(anchor, |p| yakui::Rect::from_pos_size(p, Vec2::ZERO));
        let margin = if self.props.bounded_to_side { 5. } else { 8. };
        let requested_width = if self.props.options.width > 0. {
            self.props.options.width
        } else {
            anchor.size().x.max(self.props.minimum_width)
        };
        let width = requested_width.min(viewport.x - 2. * margin).max(0.);
        for &child in &ctx.dom.get_current().children {
            let mut options = self.props.options;
            let vertical = matches!(options.side, Side::Top | Side::Bottom);
            // Select aligns a row by shifting the popup upward, while its list
            // height still comes from ordinary space below the trigger. Keeping
            // these offsets separate prevents alignment from enlarging the list.
            let height_offset = self.props.height_offset.unwrap_or(options.side_offset);
            let above = (anchor.pos().y - height_offset - margin).max(0.);
            let below = (viewport.y - anchor.max().y - height_offset - margin).max(0.);
            let (preferred, opposite) = if matches!(options.side, Side::Bottom) {
                (below, above)
            } else {
                (above, below)
            };
            let bounded = self.props.bounded_to_side && vertical;
            let available = if bounded
                && self.props.height_offset.is_none()
                && self.last_height.get() >= preferred
                && opposite > preferred
            {
                options.side = if matches!(options.side, Side::Bottom) {
                    Side::Top
                } else {
                    Side::Bottom
                };
                opposite
            } else {
                preferred
            };
            // Use one layout per widget, as required by Yakui. The previous frame's
            // clipped height tells us whether the preferred side was full. Giving
            // scrolling content its real viewport on its first pass also preserves
            // Yakui's focus-reveal event and avoids duplicate pointer registrations.
            let size = ctx.calculate_layout(
                child,
                Constraints {
                    min: Vec2::new(if self.props.fit_width { 0. } else { width }, 0.),
                    max: Vec2::new(
                        width,
                        if bounded {
                            available
                        } else {
                            (viewport.y - 2. * margin).max(0.)
                        },
                    ),
                },
            );
            self.last_height.set(size.y);
            let (start, end, extent) = if vertical {
                (anchor.pos().x, anchor.max().x, size.x)
            } else {
                (anchor.pos().y, anchor.max().y, size.y)
            };
            let reversed = vertical && self.props.rtl;
            let cross = match options.align {
                PopoverAlign::Start => {
                    if reversed {
                        end - extent - options.align_offset
                    } else {
                        start + options.align_offset
                    }
                }
                PopoverAlign::Center => (start + end - extent) / 2. + options.align_offset,
                PopoverAlign::End => {
                    if reversed {
                        start + options.align_offset
                    } else {
                        end - extent - options.align_offset
                    }
                }
            };
            let mut pos = match options.side {
                Side::Right => Vec2::new(anchor.max().x + options.side_offset, cross),
                Side::Left => Vec2::new(anchor.pos().x - size.x - options.side_offset, cross),
                Side::Top => Vec2::new(cross, anchor.pos().y - size.y - options.side_offset),
                Side::Bottom => Vec2::new(cross, anchor.max().y + options.side_offset),
            };
            match options.side {
                Side::Right if pos.x + size.x > viewport.x - 8. => {
                    pos.x = anchor.pos().x - size.x - options.side_offset
                }
                Side::Left if pos.x < 8. => pos.x = anchor.max().x + options.side_offset,
                Side::Bottom
                    if !self.props.bounded_to_side && pos.y + size.y > viewport.y - margin =>
                {
                    pos.y = anchor.pos().y - size.y - options.side_offset
                }
                Side::Top if !self.props.bounded_to_side && pos.y < margin => {
                    pos.y = anchor.max().y + options.side_offset
                }
                _ => {}
            }
            pos = pos.clamp(
                Vec2::splat(margin),
                (viewport - size - Vec2::splat(margin)).max(Vec2::splat(margin)),
            );
            ctx.layout.set_pos(child, pos);
        }
        viewport
    }
}
#[track_caller]
fn popup(
    state: &mut OverlayState,
    options: PopoverOptions,
    interactive: bool,
    auto_focus: bool,
    padding: f32,
    radius_scale: f32,
    bounded_to_side: bool,
    children: impl FnOnce(&mut OverlayState),
) {
    if state.dismiss.take() {
        state.open = false;
    }
    let props = FocusProps {
        open: state.open,
        modal: false,
        auto_focus,
        backdrop: interactive,
        trigger: state.trigger,
        dismiss: state.dismiss.clone(),
    };
    let placement = PopupPlacement {
        anchor: state.trigger,
        point: state.anchor_position,
        options,
        rtl: crate::advanced_layout::rtl(),
        bounded_to_side,
        minimum_width: yakui::util::read_scope::<PopupSizing>()
            .map_or(128., |sizing| sizing.minimum_width),
        height_offset: yakui::util::read_scope::<PopupSizing>()
            .and_then(|sizing| sizing.height_offset),
        fit_width: false,
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
                                    t.radius * radius_scale,
                                    padding,
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
    popover_with_options(
        state,
        PopoverOptions {
            width,
            ..Default::default()
        },
        children,
    )
}
/// Informational popup: content sizes naturally up to the supplied maximum.
/// It neither moves keyboard focus nor installs an outside-click backdrop.
#[track_caller]
pub(crate) fn tooltip_popup(
    trigger: ControlResponse,
    options: PopoverOptions,
    children: impl FnOnce(),
) {
    yakui::util::widget_children::<PopupPlacementWidget, _>(
        children,
        PopupPlacement {
            anchor: trigger.id,
            point: None,
            options,
            rtl: crate::advanced_layout::rtl(),
            bounded_to_side: false,
            minimum_width: 0.,
            height_offset: None,
            fit_width: true,
        },
    );
}
/// A hover-card surface. Caller controls visibility so content can remain open when hovered.
#[track_caller]
pub fn hover_card(state: &mut OverlayState, width: f32, children: impl FnOnce(&mut OverlayState)) {
    popup(
        state,
        PopoverOptions {
            width,
            ..Default::default()
        },
        false,
        false,
        8.,
        1.,
        false,
        children,
    )
}

/// Popup with a preferred side. Placement flips/clamps against the viewport.
#[track_caller]
pub fn popover_on_side(
    state: &mut OverlayState,
    width: f32,
    side: Side,
    children: impl FnOnce(&mut OverlayState),
) {
    popover_with_options(
        state,
        PopoverOptions {
            width,
            side,
            ..Default::default()
        },
        children,
    )
}

/// Popup with explicit side, alignment and offsets; render after its trigger.
#[track_caller]
pub fn popover_with_options(
    state: &mut OverlayState,
    options: PopoverOptions,
    children: impl FnOnce(&mut OverlayState),
) {
    popup(state, options, true, true, 8., 1., false, children)
}
/// A compact heading and description inside a popover.
#[track_caller]
pub fn popover_header(children: impl FnOnce()) {
    crate::column(2., children);
}
/// Medium-weight popover heading, using the body size and line height.
#[track_caller]
pub fn popover_title(text: impl Into<String>) {
    let t = crate::theme();
    yakui::widgets::RenderText::with_style(text, t.text_style(t.font_size, t.foreground, 500))
        .show();
}
/// Secondary text describing the popover's purpose.
#[track_caller]
pub fn popover_description(text: impl Into<String>) {
    crate::muted(text);
}
#[track_caller]
pub(crate) fn menu_popup(
    state: &mut OverlayState,
    options: PopoverOptions,
    auto_focus: bool,
    dismiss_on_outside: bool,
    children: impl FnOnce(&mut OverlayState),
) {
    popup(
        state,
        options,
        dismiss_on_outside,
        auto_focus,
        4.,
        0.8,
        true,
        children,
    )
}

// Navigation flyouts leave the underlying bar reachable. An outside interest on
// the popup closes it without a fullscreen pointer-blocking backdrop, so a click
// can switch to another trigger in the same input cycle.
#[derive(Debug)]
struct OutsideDismiss(Option<Rc<Cell<bool>>>);
impl Widget for OutsideDismiss {
    type Props<'a> = Rc<Cell<bool>>;
    type Response = ();
    fn new() -> Self {
        Self(None)
    }
    fn update(&mut self, dismiss: Self::Props<'_>) {
        self.0 = Some(dismiss);
    }
    fn event_interest(&self) -> yakui::event::EventInterest {
        yakui::event::EventInterest::MOUSE_OUTSIDE
    }
    fn event(
        &mut self,
        _: yakui::widget::EventContext<'_>,
        event: &yakui::event::WidgetEvent,
    ) -> yakui::event::EventResponse {
        if matches!(
            event,
            yakui::event::WidgetEvent::MouseButtonChanged {
                button: yakui::input::MouseButton::One,
                down: true,
                inside: false,
                ..
            }
        ) {
            self.0.as_ref().unwrap().set(true);
        }
        yakui::event::EventResponse::Bubble
    }
}
#[track_caller]
pub(crate) fn navigation_popup(
    state: &mut OverlayState,
    width: f32,
    keyboard_open: bool,
    children: impl FnOnce(),
) {
    let dismiss = state.dismiss.clone();
    popup(
        state,
        PopoverOptions {
            width,
            side_offset: 8.,
            ..Default::default()
        },
        false,
        keyboard_open,
        4.,
        1.,
        false,
        |_| {
            yakui::util::widget_children::<OutsideDismiss, _>(children, dismiss);
        },
    );
}

#[derive(Debug)]
struct PopupSizing {
    minimum_width: f32,
    height_offset: Option<f32>,
}
#[track_caller]
pub(crate) fn select_popup(
    state: &mut OverlayState,
    options: PopoverOptions,
    aligned: bool,
    children: impl FnOnce(&mut OverlayState),
) {
    Scope::new(PopupSizing {
        minimum_width: 144.,
        height_offset: aligned.then_some(4.),
    })
    .show(|| popup(state, options, true, false, 4., 1., true, children));
}
