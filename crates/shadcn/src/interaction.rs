//! One pointer/keyboard activation contract shared by styled controls.
use yakui::{Color, Rect, Vec2};
use yakui_core::{
    WidgetId, context,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::{KeyCode, MouseButton},
    widget::{EventContext, FocusPolicy, PaintContext, Widget},
};

/// Response from a button or another activatable control.
#[derive(Debug, Default, Clone, Copy)]
pub struct ControlResponse {
    pub clicked: bool,
    pub hovering: bool,
    pub focused: bool,
    /// Keyboard navigation focus, used to show a visible focus indicator.
    pub focus_visible: bool,
    pub pressed: bool,
    pub id: Option<WidgetId>,
}
#[derive(Debug, Clone, Copy)]
pub(crate) struct Control {
    pub disabled: bool,
    pub radius: f32,
    pub ring: Color,
    pub focus_size: Option<Vec2>,
}
impl Default for Control {
    fn default() -> Self {
        Self {
            disabled: false,
            radius: 8.0,
            ring: Color::GRAY,
            focus_size: None,
        }
    }
}
#[derive(Debug, Default)]
pub(crate) struct ControlWidget {
    props: Control,
    dismiss: Option<std::rc::Rc<crate::overlay::DismissContext>>,
    carousel: Option<std::rc::Rc<crate::carousel::CarouselContext>>,
    menu_trigger: Option<std::rc::Rc<crate::menu::MenuTriggerContext>>,
    tooltip: Option<std::rc::Rc<crate::tooltip::TooltipTriggerContext>>,
    hovering: bool,
    pointer_down: bool,
    key_down: Option<KeyCode>,
    focused: bool,
    focus_visible: bool,
    clicked: bool,
    ring_animation: Option<crate::animation::Transition<f32>>,
    ring_opacity: f32,
}
impl Widget for ControlWidget {
    type Props<'a> = Control;
    type Response = ControlResponse;
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, props: Control) -> ControlResponse {
        self.props = props;
        self.props.ring = if props.ring.a == 0 {
            props.ring
        } else {
            crate::layout::resolve_color(props.ring.with_alpha(0.5))
        };
        // Event dispatch holds a mutable DOM borrow. Capture scopes during update
        // rather than trying to borrow the current node again inside an event.
        self.dismiss = yakui::util::read_scope::<crate::overlay::DismissContext>();
        self.carousel = yakui::util::read_scope();
        self.menu_trigger = yakui::util::read_scope();
        self.tooltip = yakui::util::read_scope();
        if props.disabled {
            self.pointer_down = false;
            self.key_down = None;
            self.clicked = false;
        }
        let target = if self.focus_visible && !props.disabled {
            1.
        } else {
            0.
        };
        self.ring_opacity = crate::animation::update_transition(
            &mut self.ring_animation,
            target,
            Default::default(),
        );
        ControlResponse {
            clicked: std::mem::take(&mut self.clicked),
            hovering: self.hovering,
            focused: self.focused,
            focus_visible: self.focus_visible,
            pressed: self.pointer_down || self.key_down.is_some(),
            id: None,
        }
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        if self.ring_opacity > 0. && self.props.ring.a != 0 {
            let mut rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            if let Some(size) = self.props.focus_size {
                rect = Rect::from_pos_size(
                    rect.pos() + Vec2::new(0., (rect.size().y - size.y) / 2.),
                    size,
                );
            }
            let outer = Rect::from_pos_size(
                rect.pos() - Vec2::splat(3.0),
                rect.size() + Vec2::splat(6.0),
            );
            ctx.paint.with_opacity(self.ring_opacity, |paint| {
                crate::icons::outline(
                    &mut PaintContext {
                        dom: ctx.dom,
                        layout: ctx.layout,
                        paint,
                    },
                    outer,
                    self.props.radius + 3.,
                    3.,
                    self.props.ring,
                );
            });
        }
        for &child in &ctx.dom.get_current().children {
            ctx.paint(child);
        }
    }
    fn event_interest(&self) -> EventInterest {
        if self.props.disabled {
            EventInterest::empty()
        } else {
            EventInterest::MOUSE_INSIDE
                | EventInterest::MOUSE_OUTSIDE
                | EventInterest::FOCUSED_KEYBOARD
        }
    }
    fn focus_policy(&self) -> FocusPolicy {
        if self.props.disabled {
            FocusPolicy::empty()
        } else {
            FocusPolicy::SEQUENTIAL | FocusPolicy::DIRECTIONAL | FocusPolicy::POINTER
        }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let Some(tooltip) = &self.tooltip {
            if tooltip.escape(event) {
                return EventResponse::Sink;
            }
        }
        if self.props.disabled {
            return EventResponse::Bubble;
        }
        if crate::overlay::escape(&ctx, event, self.dismiss.as_deref()) {
            return EventResponse::Sink;
        }
        if let Some(trigger) = &self.menu_trigger {
            if let Some(response) = trigger.key(event) {
                return response;
            }
        }
        if let Some(carousel) = &self.carousel {
            if carousel.key(event) {
                return EventResponse::Sink;
            }
        }
        match event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
            }
            WidgetEvent::FocusChanged {
                focused,
                navigation,
            } => {
                self.focused = *focused;
                self.focus_visible = *focused && *navigation;
                if !focused {
                    self.key_down = None;
                    self.pointer_down = false;
                }
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                ..
            } => {
                if *down {
                    self.pointer_down = *inside;
                    if *inside {
                        ctx.input.set_focus(Some(ctx.dom.current()), false);
                        return EventResponse::Sink;
                    }
                } else if self.pointer_down {
                    self.pointer_down = false;
                    self.clicked |= *inside;
                    return EventResponse::Sink;
                }
            }
            WidgetEvent::KeyChanged {
                key: key @ (KeyCode::Enter | KeyCode::NumpadEnter),
                down,
                ..
            } => {
                if *down && self.key_down.is_none() {
                    self.clicked = true;
                    self.key_down = Some(*key);
                }
                if !down {
                    self.key_down = None;
                }
                return EventResponse::Sink;
            }
            WidgetEvent::KeyChanged {
                key: KeyCode::Space,
                down,
                ..
            } => {
                if *down {
                    self.key_down = Some(KeyCode::Space);
                } else {
                    self.clicked |= self.key_down == Some(KeyCode::Space);
                    self.key_down = None;
                }
                return EventResponse::Sink;
            }
            _ => return EventResponse::Bubble,
        }
        EventResponse::Bubble
    }
}
/// Build the child's appearance from the response before the layout and paint phases.
#[track_caller]
pub(crate) fn control(props: Control, children: impl FnOnce(ControlResponse)) -> ControlResponse {
    let dom = context::dom();
    let response = dom.begin_widget::<ControlWidget>(props);
    let id = response.id;
    let output = ControlResponse {
        id: Some(id),
        ..response.into_inner()
    };
    children(output);
    dom.end_widget::<ControlWidget>(id);
    output
}

#[derive(Clone, Default)]
struct PointerDragRequest(std::rc::Rc<std::cell::Cell<bool>>);

pub(crate) fn request_pointer_drag(dom: &yakui_core::dom::Dom) {
    dom.get_global_or_init(PointerDragRequest::default)
        .0
        .set(true);
}

/// Consume a drag request after dispatching a pointer press. Touch hosts use this
/// to leave slider/split-handle gestures with the control instead of scrolling.
pub fn take_pointer_drag_request(yak: &yakui::Yakui) -> bool {
    yak.dom()
        .get_global_or_init(PointerDragRequest::default)
        .0
        .replace(false)
}
