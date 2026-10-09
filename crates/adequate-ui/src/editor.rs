//! Delegate text editing to Yakui while allowing root overlays to own Escape and Tab.
use yakui::{
    Constraints, Vec2,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::KeyCode,
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
    widgets::{TextBox, TextBoxResponse, TextBoxWidget},
};
#[derive(Debug)]
pub(crate) struct Editor {
    inner: TextBoxWidget,
    focused: bool,
    radius: f32,
    ring: yakui::Color,
    dismiss: Option<std::rc::Rc<crate::overlay::DismissContext>>,
}
impl Widget for Editor {
    type Props<'a> = TextBox;
    type Response = TextBoxResponse;
    fn new() -> Self {
        Self {
            inner: TextBoxWidget::new(),
            focused: false,
            radius: 10.,
            ring: yakui::Color::GRAY,
            dismiss: None,
        }
    }
    fn update(&mut self, props: TextBox) -> TextBoxResponse {
        self.dismiss = yakui::util::read_scope::<crate::overlay::DismissContext>();
        self.radius = props.radius;
        self.ring = crate::layout::resolve_color(crate::theme().ring.with_alpha(0.5));
        self.inner.update(props)
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        self.inner.layout(ctx, constraints)
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        if self.focused {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            let outer = yakui::Rect::from_pos_size(
                rect.pos() - Vec2::splat(3.),
                rect.size() + Vec2::splat(6.),
            );
            yakui::shapes::RoundedRectangle::new(outer, self.radius + 3.)
                .color(yakui::Color::CLEAR)
                .border(Some(yakui::Border::new(self.ring, 3.)))
                .add(ctx.paint);
        }
        self.inner.paint(ctx)
    }
    fn event_interest(&self) -> EventInterest {
        self.inner.event_interest()
    }
    fn focus_policy(&self) -> FocusPolicy {
        self.inner.focus_policy()
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::FocusChanged { focused, .. } = event {
            self.focused = *focused;
        }
        if crate::overlay::escape(&ctx, event, self.dismiss.as_deref()) {
            return EventResponse::Sink;
        }
        if matches!(
            event,
            WidgetEvent::KeyChanged {
                key: KeyCode::Tab,
                ..
            }
        ) {
            return EventResponse::Bubble;
        }
        self.inner.event(ctx, event)
    }
}
