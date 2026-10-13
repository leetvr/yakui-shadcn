//! Native carousel navigation and swipe state with explicit slide composition.
use crate::{Button, ButtonSize, Icon, Orientation, Variant};
use std::{cell::Cell, rc::Rc, time::Duration};
use web_time::Instant;
use yakui::{
    Constraints, Vec2,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::{KeyCode, MouseButton},
    widget::{EventContext, FocusPolicy, LayoutContext, Widget},
};
#[derive(Debug, Clone)]
pub struct CarouselState {
    pub index: usize,
    pub looping: bool,
    last_transition: Instant,
    last_step: Option<(usize, i32)>,
}
impl Default for CarouselState {
    fn default() -> Self {
        Self {
            index: 0,
            looping: false,
            last_transition: Instant::now(),
            last_step: None,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct CarouselOptions {
    pub size: Vec2,
    pub orientation: Orientation,
    pub visible: usize,
    pub autoplay: Option<Duration>,
}
impl Default for CarouselOptions {
    fn default() -> Self {
        Self {
            size: Vec2::splat(320.),
            orientation: Orientation::Horizontal,
            visible: 1,
            autoplay: None,
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct CarouselContext {
    pending: Rc<Cell<i32>>,
    orientation: Orientation,
}
impl CarouselContext {
    pub(crate) fn key(&self, event: &WidgetEvent) -> bool {
        if let WidgetEvent::KeyChanged {
            key, down: true, ..
        } = event
        {
            let delta = match (self.orientation, key) {
                (Orientation::Horizontal, KeyCode::ArrowLeft)
                | (Orientation::Vertical, KeyCode::ArrowUp) => -1,
                (Orientation::Horizontal, KeyCode::ArrowRight)
                | (Orientation::Vertical, KeyCode::ArrowDown) => 1,
                _ => return false,
            };
            self.pending.set(delta);
            return true;
        }
        false
    }
}
#[derive(Debug)]
struct Swipe {
    scope: Option<Rc<CarouselContext>>,
    start: Option<Vec2>,
}
impl Widget for Swipe {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self {
            scope: None,
            start: None,
        }
    }
    fn update(&mut self, _: ()) {
        self.scope = yakui::util::read_scope();
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        self.default_layout(
            LayoutContext {
                dom: ctx.dom,
                input: ctx.input,
                layout: ctx.layout,
                paint: ctx.paint,
            },
            constraints,
        )
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL | EventInterest::FOCUSED_KEYBOARD
    }
    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::SEQUENTIAL | FocusPolicy::POINTER
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        let Some(scope) = &self.scope else {
            return EventResponse::Bubble;
        };
        if scope.key(event) {
            return EventResponse::Sink;
        }
        match event {
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                position,
                ..
            } => {
                self.start = Some(*position);
                ctx.input.set_focus(Some(ctx.dom.current()), false);
                return EventResponse::Sink;
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: false,
                position,
                ..
            } => {
                if let Some(start) = self.start.take() {
                    let delta = if scope.orientation == Orientation::Horizontal {
                        position.x - start.x
                    } else {
                        position.y - start.y
                    };
                    if delta.abs() >= 40. {
                        scope.pending.set(if delta < 0. { 1 } else { -1 });
                    }
                    return EventResponse::Sink;
                }
            }
            _ => {}
        }
        EventResponse::Bubble
    }
}
#[derive(Debug, Clone, Copy)]
struct TrackProps {
    index: usize,
    visible: usize,
    options: CarouselOptions,
    count: usize,
    looping: bool,
    step: i32,
}
#[derive(Debug)]
struct Track {
    props: TrackProps,
    transition: Option<crate::animation::Transition<f32>>,
    position: f32,
    target: f32,
}
impl Widget for Track {
    type Props<'a> = TrackProps;
    type Response = ();
    fn new() -> Self {
        Self {
            props: TrackProps {
                index: 0,
                visible: 1,
                options: Default::default(),
                count: 0,
                looping: false,
                step: 0,
            },
            transition: None,
            position: 0.,
            target: 0.,
        }
    }
    fn update(&mut self, props: TrackProps) {
        if self.transition.is_none()
            || self.props.count != props.count
            || self.props.looping != props.looping
        {
            self.target = props.index as f32;
            self.transition = None;
        } else if self.props.index != props.index {
            if props.looping {
                // Keep an unwrapped coordinate across the seam. Slides are laid
                // out periodically, once each, so their retained input/editor
                // state never migrates to a duplicate copy during a wrap.
                let count = props.count as f32;
                let delta = props.index as f32 - self.props.index as f32;
                let delta = if props.step > 0 {
                    delta.rem_euclid(count)
                } else if props.step < 0 {
                    -(-delta).rem_euclid(count)
                } else {
                    (delta + count * 0.5).rem_euclid(count) - count * 0.5
                };
                self.target += delta;
            } else {
                self.target = props.index as f32;
            }
        }
        self.props = props;
        let target = self.target;
        let now = crate::animation::now();
        let transition = self
            .transition
            .get_or_insert_with(|| crate::animation::Transition::new(target));
        if crate::animation::enabled() {
            transition.set_target(
                target,
                crate::animation::Motion::new(
                    Duration::from_millis(300),
                    crate::animation::Easing::CubicBezier([0.22, 1., 0.36, 1.]),
                ),
                now,
            );
        } else {
            transition.snap(target);
        }
        self.position = transition.value(now);
        if transition.is_active(now) {
            crate::animation::request_frame();
        }
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let horizontal = self.props.options.orientation == Orientation::Horizontal;
        let size = constraints.constrain(self.props.options.size);
        let slot = if horizontal {
            Vec2::new(size.x / self.props.visible as f32, size.y)
        } else {
            Vec2::new(size.x, size.y / self.props.visible as f32)
        };
        for (index, &child) in ctx.dom.get_current().children.iter().enumerate() {
            ctx.calculate_layout(child, Constraints::tight(slot));
            let mut offset = index as f32 - self.position;
            if self.props.looping {
                // Relocate a slide only outside the clipped viewport. At least
                // one spare slot is required to hide that relocation.
                offset = (offset + 1.).rem_euclid(self.props.count as f32) - 1.;
            }
            ctx.layout.set_pos(
                child,
                if horizontal {
                    Vec2::new(offset * slot.x, 0.)
                } else {
                    Vec2::new(0., offset * slot.y)
                },
            );
            if !self.selected(index) {
                // The viewport clips paint and pointer bounds. Logical selection
                // also excludes outgoing/offscreen slides from keyboard input.
                let mut focus = ctx.input.focus();
                while let Some(id) = focus {
                    if id == child {
                        ctx.input
                            .set_focus(ctx.dom.get(ctx.dom.current()).unwrap().parent, true);
                        break;
                    }
                    focus = ctx.dom.get(id).and_then(|node| node.parent);
                }
                ctx.layout.disable_input(ctx.dom, child);
            }
        }
        size
    }
    fn navigate(
        &self,
        ctx: yakui::widget::NavigateContext<'_>,
        direction: yakui_core::navigation::NavDirection,
    ) -> Option<yakui::WidgetId> {
        let children = &ctx.dom.get_current().children;
        let selected =
            (0..self.props.visible).map(|slot| (self.props.index + slot) % self.props.count);
        if matches!(direction, yakui_core::navigation::NavDirection::Previous) {
            selected
                .rev()
                .find_map(|index| ctx.try_navigate(children[index], direction))
        } else {
            selected
                .into_iter()
                .find_map(|index| ctx.try_navigate(children[index], direction))
        }
    }
}
impl Track {
    fn selected(&self, index: usize) -> bool {
        (index + self.props.count - self.props.index) % self.props.count < self.props.visible
    }
}
fn advance(state: &mut CarouselState, count: usize, visible: usize, delta: i32) {
    let max = count.saturating_sub(visible);
    state.index = if count == visible {
        0
    } else if state.looping {
        ((state.index as i64 + delta as i64).rem_euclid(count.max(1) as i64)) as usize
    } else {
        (state.index as i64 + delta as i64).clamp(0, max as i64) as usize
    };
    state.last_transition = Instant::now();
    state.last_step = Some((state.index, delta));
}
/// Render a native slide track. Slides stay declared for smooth movement and
/// retain their state; offscreen slides ignore input and keyboard navigation.
/// Arrows/swipe update app-owned selection. `looping` enables cyclic navigation.
#[track_caller]
pub fn carousel(
    state: &mut CarouselState,
    count: usize,
    options: CarouselOptions,
    mut slide: impl FnMut(usize),
) {
    if count == 0 {
        return;
    }
    let visible = options.visible.max(1).min(count);
    let looping = state.looping && visible < count;
    state.index = state
        .index
        .min(if looping { count - 1 } else { count - visible });
    crate::layout::with_state(
        || Rc::new(Cell::new(0)),
        |hook| {
            let pending = hook.borrow().clone();
            let delta = pending.replace(0);
            if delta != 0 {
                advance(state, count, visible, delta);
            }
            if options
                .autoplay
                .is_some_and(|delay| state.last_transition.elapsed() >= delay)
            {
                advance(state, count, visible, 1);
            }
            let context = CarouselContext {
                pending,
                orientation: options.orientation,
            };
            yakui::widgets::Scope::new(context).show(|| {
                let body = || {
                    if Button::new("")
                        .icon(Icon::ChevronLeft)
                        .variant(Variant::Outline)
                        .size(ButtonSize::Icon)
                        .rounded(true)
                        .disabled(count == visible || (!looping && state.index == 0))
                        .show()
                        .clicked
                    {
                        advance(state, count, visible, -1);
                    }
                    crate::size(options.size, || {
                        yakui::util::widget_children::<Swipe, _>(
                            || {
                                yakui::util::widget_children::<Track, _>(
                                    || {
                                        for index in 0..count {
                                            yakui::widgets::ConstrainedBox::new(
                                                Constraints::none(),
                                            )
                                            .show(|| slide(index));
                                        }
                                    },
                                    TrackProps {
                                        index: state.index,
                                        visible,
                                        options,
                                        count,
                                        looping,
                                        step: state
                                            .last_step
                                            .take()
                                            .filter(|(index, _)| *index == state.index)
                                            .map_or(0, |(_, step)| step),
                                    },
                                );
                            },
                            (),
                        );
                    });
                    if Button::new("")
                        .icon(Icon::ChevronRight)
                        .variant(Variant::Outline)
                        .size(ButtonSize::Icon)
                        .rounded(true)
                        .disabled(count == visible || (!looping && state.index >= count - visible))
                        .show()
                        .clicked
                    {
                        advance(state, count, visible, 1);
                    }
                };
                if options.orientation == Orientation::Horizontal {
                    crate::row(16., body)
                } else {
                    crate::column(16., body)
                }
            });
        },
    )
}
