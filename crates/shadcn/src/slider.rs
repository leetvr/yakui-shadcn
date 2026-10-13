//! Ordered native slider thumbs with pointer capture and keyboard adjustment.
use yakui::{
    Color, Constraints, Rect, Vec2,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::{KeyCode, MouseButton},
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
};
/// Axis shared by sliders, carousels and split panels.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}
#[derive(Debug, Clone, Copy)]
pub struct SliderOptions {
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub length: f32,
    pub orientation: Orientation,
    pub disabled: bool,
}
impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            min: 0.,
            max: 100.,
            step: 1.,
            length: 240.,
            orientation: Orientation::Horizontal,
            disabled: false,
        }
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct SliderResponse {
    pub changed: bool,
    pub confirmed: bool,
    pub focused: bool,
}
#[derive(Debug, Clone)]
struct SliderProps {
    values: Vec<f64>,
    options: SliderOptions,
}
#[derive(Debug)]
struct SliderWidget {
    props: SliderProps,
    pending: Option<Vec<f64>>,
    confirmed: bool,
    dragging: bool,
    active: usize,
    focused: bool,
    focus_visible: bool,
    hovering: Option<usize>,
    ring_transitions: Vec<Option<crate::animation::Transition<f32>>>,
    ring_widths: Vec<f32>,
    track: Color,
    fill: Color,
    ring: Color,
    border: Color,
    thumb: Color,
    dismiss: Option<std::rc::Rc<crate::overlay::DismissContext>>,
}
impl SliderWidget {
    fn set_value(&mut self, value: f64) {
        if self.props.values.is_empty() {
            return;
        }
        let o = self.props.options;
        let low = self
            .active
            .checked_sub(1)
            .map_or(o.min, |i| self.props.values[i]);
        let high = self
            .props
            .values
            .get(self.active + 1)
            .copied()
            .unwrap_or(o.max);
        let value = (o.min + ((value - o.min) / o.step).round() * o.step).clamp(low, high);
        self.props.values[self.active] = value;
        self.pending = Some(self.props.values.clone());
    }
    fn thumb_center(&self, rect: Rect, value: f64) -> Vec2 {
        let o = self.props.options;
        let ratio = ((value - o.min) / (o.max - o.min).max(f64::EPSILON)) as f32;
        if o.orientation == Orientation::Horizontal {
            Vec2::new(
                rect.pos().x + 6. + ratio * (rect.size().x - 12.).max(0.),
                rect.center().y,
            )
        } else {
            Vec2::new(
                rect.center().x,
                rect.max().y - 6. - ratio * (rect.size().y - 12.).max(0.),
            )
        }
    }
    fn hovered_thumb(&self, ctx: &EventContext<'_>, position: Vec2) -> Option<usize> {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        self.props
            .values
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, &value)| {
                // Upstream expands the 12px thumb's pointer target by 8px per side.
                let delta = (position - self.thumb_center(rect, value)).abs();
                (delta.max_element() <= 14.).then_some(i)
            })
    }
    fn pointer_value(&self, ctx: &EventContext<'_>, position: Vec2) -> f64 {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let ratio = if self.props.options.orientation == Orientation::Horizontal {
            (position.x - rect.pos().x - 6.) / (rect.size().x - 12.).max(1.)
        } else {
            1. - (position.y - rect.pos().y - 6.) / (rect.size().y - 12.).max(1.)
        };
        let o = self.props.options;
        o.min + ratio.clamp(0., 1.) as f64 * (o.max - o.min)
    }
}
impl Widget for SliderWidget {
    type Props<'a> = SliderProps;
    type Response = (Option<Vec<f64>>, bool, bool);
    fn new() -> Self {
        Self {
            props: SliderProps {
                values: vec![],
                options: SliderOptions::default(),
            },
            pending: None,
            confirmed: false,
            dragging: false,
            active: 0,
            focused: false,
            focus_visible: false,
            hovering: None,
            ring_transitions: Vec::new(),
            ring_widths: Vec::new(),
            track: Color::GRAY,
            fill: Color::BLACK,
            ring: Color::GRAY,
            border: Color::GRAY,
            thumb: Color::WHITE,
            dismiss: None,
        }
    }
    fn update(&mut self, props: SliderProps) -> Self::Response {
        self.dismiss = yakui::util::read_scope();
        let pending = self.pending.take();
        self.props = props;
        // Events update the retained values before the application reads this
        // response. Paint those same values now, avoiding a frame of drag latency.
        if let Some(values) = &pending {
            for (value, next) in self.props.values.iter_mut().zip(values) {
                *value = *next;
            }
        }
        self.active = self.active.min(self.props.values.len().saturating_sub(1));
        let t = crate::theme();
        let color = |color: Color| {
            crate::layout::resolve_color(if self.props.options.disabled {
                color.with_alpha(0.5)
            } else {
                color
            })
        };
        self.track = color(t.muted);
        self.fill = color(t.primary);
        self.border = color(t.ring);
        self.thumb = color(Color::WHITE);
        self.ring = crate::layout::resolve_color(t.ring.with_alpha(0.5));
        if self.props.options.disabled {
            self.dragging = false;
            self.hovering = None;
        }
        self.ring_transitions.resize(self.props.values.len(), None);
        self.ring_widths = self
            .ring_transitions
            .iter_mut()
            .enumerate()
            .map(|(i, transition)| {
                let visible = !self.props.options.disabled
                    && (self.hovering == Some(i)
                        || (i == self.active && (self.focus_visible || self.dragging)));
                crate::animation::update_transition(
                    transition,
                    if visible { 3. } else { 0. },
                    Default::default(),
                )
            })
            .collect();
        (pending, std::mem::take(&mut self.confirmed), self.focused)
    }
    fn layout(&self, _ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        constraints.constrain(
            if self.props.options.orientation == Orientation::Horizontal {
                Vec2::new(self.props.options.length, 20.)
            } else {
                Vec2::new(20., self.props.options.length)
            },
        )
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let o = self.props.options;
        let horizontal = o.orientation == Orientation::Horizontal;
        let point = |value| self.thumb_center(rect, value);
        let track = if horizontal {
            Rect::from_pos_size(
                Vec2::new(rect.pos().x, rect.center().y - 2.),
                Vec2::new(rect.size().x, 4.),
            )
        } else {
            Rect::from_pos_size(
                Vec2::new(rect.center().x - 2., rect.pos().y),
                Vec2::new(4., rect.size().y),
            )
        };
        yakui::shapes::RoundedRectangle::new(track, 999.)
            .color(self.track)
            .add(ctx.paint);
        if let (Some(first), Some(last)) = (self.props.values.first(), self.props.values.last()) {
            let start = if self.props.values.len() == 1 {
                if horizontal { track.pos() } else { track.max() }
            } else {
                point(*first)
            };
            let end = point(*last);
            let fill = if horizontal {
                Rect::from_pos_size(
                    Vec2::new(start.x, track.pos().y),
                    Vec2::new((end.x - start.x).max(0.), 4.),
                )
            } else {
                Rect::from_pos_size(
                    Vec2::new(track.pos().x, end.y),
                    Vec2::new(4., (start.y - end.y).max(0.)),
                )
            };
            yakui::shapes::RoundedRectangle::new(fill, 999.)
                .color(self.fill)
                .add(ctx.paint);
        }
        for (i, value) in self.props.values.iter().enumerate() {
            let center = point(*value);
            let ring_width = self.ring_widths[i];
            if ring_width > 0. {
                crate::icons::circle(
                    &mut ctx,
                    center,
                    6. + ring_width / 2.,
                    ring_width,
                    self.ring,
                    0.,
                    std::f32::consts::TAU,
                );
            }
            yakui::shapes::Circle::new(center, 6.)
                .color(self.thumb)
                .add(ctx.paint);
            crate::icons::circle(
                &mut ctx,
                center,
                5.5,
                1.,
                self.border,
                0.,
                std::f32::consts::TAU,
            );
        }
    }
    fn event_interest(&self) -> EventInterest {
        if self.props.options.disabled {
            EventInterest::empty()
        } else {
            EventInterest::MOUSE_ALL | EventInterest::FOCUSED_KEYBOARD
        }
    }
    fn focus_policy(&self) -> FocusPolicy {
        if self.props.options.disabled {
            FocusPolicy::empty()
        } else {
            FocusPolicy::SEQUENTIAL | FocusPolicy::POINTER
        }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if self.props.options.disabled {
            return EventResponse::Bubble;
        }
        if crate::overlay::escape(&ctx, event, self.dismiss.as_deref()) {
            self.dragging = false;
            return EventResponse::Sink;
        }
        match event {
            WidgetEvent::MouseLeave => self.hovering = None,
            WidgetEvent::FocusChanged {
                focused,
                navigation,
            } => {
                self.focus_visible = *focused && *navigation;
                self.focused = *focused;
                if !focused {
                    self.dragging = false;
                }
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                position,
                ..
            } => {
                let value = self.pointer_value(&ctx, *position);
                self.active = self
                    .props
                    .values
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| (**a - value).abs().total_cmp(&(**b - value).abs()))
                    .map_or(0, |(i, _)| i);
                self.dragging = true;
                ctx.input.set_focus(Some(ctx.dom.current()), false);
                self.set_value(value);
                return EventResponse::Sink;
            }
            WidgetEvent::MouseMoved(position) => {
                self.hovering = position.and_then(|position| self.hovered_thumb(&ctx, position));
                if let Some(position) = position.filter(|_| self.dragging) {
                    self.set_value(self.pointer_value(&ctx, position));
                    return EventResponse::Sink;
                }
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: false,
                ..
            } if self.dragging => {
                self.dragging = false;
                self.confirmed = true;
                return EventResponse::Sink;
            }
            WidgetEvent::KeyChanged {
                key,
                down: true,
                modifiers,
            } => {
                if *key == KeyCode::Tab {
                    // One retained widget paints all thumbs, so handle the internal
                    // tab sequence before returning navigation to the parent scope.
                    if modifiers.shift() && self.active > 0 {
                        self.active -= 1;
                        return EventResponse::Sink;
                    }
                    if !modifiers.shift() && self.active + 1 < self.props.values.len() {
                        self.active += 1;
                        return EventResponse::Sink;
                    }
                    self.active = if modifiers.shift() {
                        self.props.values.len().saturating_sub(1)
                    } else {
                        0
                    };
                    return EventResponse::Bubble;
                }
                let current = self
                    .props
                    .values
                    .get(self.active)
                    .copied()
                    .unwrap_or(self.props.options.min);
                let o = self.props.options;
                let value = match key {
                    KeyCode::ArrowRight | KeyCode::ArrowUp => current + o.step,
                    KeyCode::ArrowLeft | KeyCode::ArrowDown => current - o.step,
                    KeyCode::Home => o.min,
                    KeyCode::End => o.max,
                    KeyCode::PageUp => current + o.step * 10.,
                    KeyCode::PageDown => current - o.step * 10.,
                    _ => return EventResponse::Bubble,
                };
                self.set_value(value);
                self.confirmed = true;
                return EventResponse::Sink;
            }
            _ => {}
        }
        EventResponse::Bubble
    }
}
/// A single-thumb slider. Range state is an ordinary floating-point application value.
#[track_caller]
pub fn slider(value: &mut f64, options: SliderOptions) -> SliderResponse {
    range_slider(std::slice::from_mut(value), options)
}
/// Ordered multiple thumbs. Neighbours constrain movement, so thumb identities never swap.
#[track_caller]
pub fn range_slider(values: &mut [f64], mut options: SliderOptions) -> SliderResponse {
    if !options.min.is_finite() {
        options.min = 0.;
    }
    if !options.max.is_finite() {
        options.max = 100.;
    }
    if options.min > options.max {
        std::mem::swap(&mut options.min, &mut options.max);
    }
    if !options.step.is_finite() || options.step <= 0. {
        options.step = 1.;
    }
    let mut previous = options.min;
    for value in &mut *values {
        if !value.is_finite() {
            *value = options.min;
        }
        *value = (*value).clamp(previous, options.max);
        previous = *value;
    }
    let result = yakui::util::widget::<SliderWidget>(SliderProps {
        values: values.to_vec(),
        options,
    })
    .into_inner();
    let mut changed = false;
    if let Some(next) = result.0 {
        for (value, next) in values.iter_mut().zip(next) {
            changed |= *value != next;
            *value = next;
        }
    }
    SliderResponse {
        changed,
        confirmed: result.1,
        focused: result.2,
    }
}
