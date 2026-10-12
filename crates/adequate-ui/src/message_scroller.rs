//! Chat scroll ownership: append follows the tail; prepend preserves a visible message anchor.
use std::{cell::RefCell, rc::Rc};
use yakui::{
    Constraints, Vec2, WidgetId,
    event::{EventInterest, EventResponse, WidgetEvent},
    widget::{EventContext, LayoutContext, PaintContext, Widget},
};
#[derive(Debug, Clone, Copy)]
enum ScrollCommand {
    Start,
    End,
    Message(u64),
}
#[derive(Debug)]
struct ScrollData {
    position: f32,
    extent: f32,
    viewport: f32,
    following: bool,
    anchor: Option<(u64, f32)>,
    visible: Vec<u64>,
    command: Option<ScrollCommand>,
    last_focus: Option<WidgetId>,
}
/// Retained, application-owned transcript state and navigation commands.
#[derive(Debug, Clone)]
pub struct MessageScrollerState {
    data: Rc<RefCell<ScrollData>>,
}
impl Default for MessageScrollerState {
    fn default() -> Self {
        Self {
            data: Rc::new(RefCell::new(ScrollData {
                position: 0.,
                extent: 0.,
                viewport: 0.,
                following: true,
                anchor: None,
                visible: vec![],
                command: None,
                last_focus: None,
            })),
        }
    }
}
impl MessageScrollerState {
    pub fn scroll_to_start(&mut self) {
        self.data.borrow_mut().command = Some(ScrollCommand::Start);
    }
    pub fn scroll_to_end(&mut self) {
        self.data.borrow_mut().command = Some(ScrollCommand::End);
    }
    pub fn scroll_to_message(&mut self, id: u64) {
        self.data.borrow_mut().command = Some(ScrollCommand::Message(id));
    }
    pub fn visible_message_ids(&self) -> Vec<u64> {
        self.data.borrow().visible.clone()
    }
    pub fn following(&self) -> bool {
        self.data.borrow().following
    }
    pub fn position(&self) -> f32 {
        self.data.borrow().position
    }
}
#[derive(Debug)]
struct MessageItem {
    id: u64,
}
impl Widget for MessageItem {
    type Props<'a> = u64;
    type Response = ();
    fn new() -> Self {
        Self { id: 0 }
    }
    fn update(&mut self, id: u64) {
        self.id = id;
    }
}
/// Stable IDs allow prepend/reflow to preserve the message the reader was viewing.
#[track_caller]
pub fn message_scroller_item(id: u64, children: impl FnOnce()) {
    yakui::util::widget_children::<MessageItem, _>(children, id);
}
#[derive(Debug, Clone)]
struct ScrollerProps {
    size: Vec2,
    state: MessageScrollerState,
}
#[derive(Debug)]
struct Scroller {
    props: ScrollerProps,
    bar: yakui::Color,
}
impl Widget for Scroller {
    type Props<'a> = ScrollerProps;
    type Response = ();
    fn new() -> Self {
        Self {
            props: ScrollerProps {
                size: Vec2::new(480., 320.),
                state: MessageScrollerState::default(),
            },
            bar: yakui::Color::GRAY,
        }
    }
    fn update(&mut self, props: ScrollerProps) {
        self.props = props;
        self.bar = crate::layout::resolve_color(crate::theme().border);
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        let size = constraints.constrain(self.props.size);
        let root = ctx.dom.current();
        let mut extent = 0f32;
        for &child in &ctx.dom.get_current().children {
            extent = extent.max(
                ctx.calculate_layout(
                    child,
                    Constraints {
                        min: Vec2::new(size.x, 0.),
                        max: Vec2::new(size.x, f32::INFINITY),
                    },
                )
                .y,
            );
        }
        fn collect(
            ctx: &LayoutContext<'_>,
            id: WidgetId,
            root: WidgetId,
            out: &mut Vec<(u64, yakui::Rect)>,
        ) {
            let node = ctx.dom.get(id).unwrap();
            if let Some(item) = node.widget.as_any().downcast_ref::<MessageItem>() {
                if let Some(rect) = ctx.rect_relative_to(id, root) {
                    out.push((item.id, rect));
                }
            }
            for &child in &node.children {
                collect(ctx, child, root, out);
            }
        }
        let mut items = Vec::new();
        for &child in &ctx.dom.get_current().children {
            collect(&ctx, child, root, &mut items);
        }
        let mut data = self.props.state.data.borrow_mut();
        let max = (extent - size.y).max(0.);
        data.extent = extent;
        data.viewport = size.y;
        if let Some(command) = data.command.take() {
            match command {
                ScrollCommand::Start => {
                    data.position = 0.;
                    data.following = false;
                }
                ScrollCommand::End => {
                    data.position = max;
                    data.following = true;
                }
                ScrollCommand::Message(id) => {
                    if let Some((_, rect)) = items.iter().find(|(i, _)| *i == id) {
                        data.position = rect.pos().y;
                        data.following = false;
                    }
                }
            }
            data.anchor = None;
        } else if data.following {
            data.position = max;
        } else if let Some((id, screen_y)) = data.anchor {
            if let Some((_, rect)) = items.iter().find(|(i, _)| *i == id) {
                data.position = rect.pos().y - screen_y;
            }
        }
        // Explicit keyboard focus has priority over an old reading anchor.
        let focus = ctx.input.focus();
        if focus != data.last_focus {
            if let Some(focus) = focus {
                if let Some(rect) = ctx.rect_relative_to(focus, root) {
                    if rect.pos().y < data.position {
                        data.position = rect.pos().y;
                    } else if rect.max().y > data.position + size.y {
                        data.position = rect.max().y - size.y;
                    }
                }
            }
            data.last_focus = focus;
        }
        data.position = data.position.clamp(0., max);
        data.visible.clear();
        data.anchor = None;
        for (id, rect) in items {
            if rect.max().y > data.position && rect.pos().y < data.position + size.y {
                data.visible.push(id);
                if data.anchor.is_none() {
                    data.anchor = Some((id, rect.pos().y - data.position));
                }
            }
        }
        for &child in &ctx.dom.get_current().children {
            ctx.layout.set_pos(child, Vec2::new(0., -data.position));
        }
        size
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        for &child in &ctx.dom.get_current().children {
            ctx.paint(child);
        }
        let data = self.props.state.data.borrow();
        if data.extent > data.viewport && data.viewport > 0. {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            let height = (data.viewport * data.viewport / data.extent).max(18.);
            let y = rect.pos().y
                + (rect.size().y - height) * data.position / (data.extent - data.viewport);
            yakui::shapes::RoundedRectangle::new(
                yakui::Rect::from_pos_size(Vec2::new(rect.max().x - 5., y), Vec2::new(4., height)),
                999.,
            )
            .color(self.bar)
            .add(ctx.paint);
        }
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }
    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::MouseScroll { delta, .. } = event {
            let mut data = self.props.state.data.borrow_mut();
            let max = (data.extent - data.viewport).max(0.);
            data.position = (data.position + delta.y).clamp(0., max);
            data.following = max - data.position <= 24.;
            data.anchor = None;
            return EventResponse::Sink;
        }
        EventResponse::Bubble
    }
}
/// Native transcript viewport. Render items in stable ID order inside the closure.
#[track_caller]
pub fn message_scroller(size: Vec2, state: &mut MessageScrollerState, children: impl FnOnce()) {
    yakui::util::widget_children::<Scroller, _>(
        || crate::column(12., children),
        ScrollerProps {
            size,
            state: state.clone(),
        },
    );
    if !state.following() {
        if crate::Button::new("Scroll to latest")
            .variant(crate::Variant::Outline)
            .show()
            .clicked
        {
            state.scroll_to_end();
        }
    }
}
