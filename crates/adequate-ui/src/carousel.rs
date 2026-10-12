//! Native carousel navigation and swipe state with explicit slide composition.
use crate::{Button, ButtonSize, Icon, Orientation, Variant};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};
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
}
impl Default for CarouselState {
    fn default() -> Self {
        Self {
            index: 0,
            looping: false,
            last_transition: Instant::now(),
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
fn advance(state: &mut CarouselState, count: usize, visible: usize, delta: i32) {
    let max = count.saturating_sub(visible);
    state.index = if state.looping {
        ((state.index as i64 + delta as i64).rem_euclid(count.max(1) as i64)) as usize
    } else {
        (state.index as i64 + delta as i64).clamp(0, max as i64) as usize
    };
    state.last_transition = Instant::now();
}
/// Render native slides by index. Only visible slides are built; arrows and swipe
/// update application-owned selection. Set `looping` for cyclic navigation.
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
    state.index = state.index.min(count - 1);
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
                        .disabled(!state.looping && state.index == 0)
                        .show()
                        .clicked
                    {
                        advance(state, count, visible, -1);
                    }
                    crate::size(options.size, || {
                        yakui::util::widget_children::<Swipe, _>(
                            || {
                                let mut slides = || {
                                    for offset in 0..visible {
                                        let index = (state.index + offset) % count;
                                        let size = if options.orientation == Orientation::Horizontal
                                        {
                                            Vec2::new(
                                                options.size.x / visible as f32,
                                                options.size.y,
                                            )
                                        } else {
                                            Vec2::new(
                                                options.size.x,
                                                options.size.y / visible as f32,
                                            )
                                        };
                                        crate::size(size, || slide(index));
                                    }
                                };
                                if options.orientation == Orientation::Horizontal {
                                    crate::row(0., &mut slides)
                                } else {
                                    crate::column(0., &mut slides)
                                }
                            },
                            (),
                        );
                    });
                    if Button::new("")
                        .icon(Icon::ChevronRight)
                        .variant(Variant::Outline)
                        .size(ButtonSize::Icon)
                        .rounded(true)
                        .disabled(!state.looping && state.index >= count - visible)
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
