//! Cosmic Text owns editing; this wrapper owns themed placeholders, disabled state,
//! and the surrounding overlay's keyboard contract.
use yakui::{
    Constraints, Vec2,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::KeyCode,
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
    widgets::{RenderText, TextBox, TextBoxResponse, TextBoxWidget},
};
#[derive(Debug)]
pub(crate) struct EditorProps {
    pub textbox: TextBox,
    pub disabled: bool,
    pub read_only: bool,
    pub invalid: bool,
}
#[derive(Debug)]
pub(crate) struct Editor {
    inner: TextBoxWidget,
    focused: bool,
    radius: f32,
    ring: yakui::Color,
    disabled: bool,
    grouped: bool,
    read_only: bool,
    menu: Option<std::rc::Rc<crate::menu::MenuContext>>,
    dismiss: Option<std::rc::Rc<crate::overlay::DismissContext>>,
}
impl Widget for Editor {
    type Props<'a> = EditorProps;
    type Response = TextBoxResponse;
    fn new() -> Self {
        Self {
            inner: TextBoxWidget::new(),
            focused: false,
            radius: 10.,
            ring: yakui::Color::GRAY,
            disabled: false,
            grouped: false,
            read_only: false,
            menu: None,
            dismiss: None,
        }
    }
    fn update(&mut self, mut props: EditorProps) -> TextBoxResponse {
        self.dismiss = yakui::util::read_scope();
        self.menu = yakui::util::read_scope();
        self.disabled = props.disabled;
        self.grouped = yakui::util::read_scope::<crate::input_group::InputGroupContext>().is_some();
        self.read_only = props.read_only;
        self.radius = props.textbox.radius;
        let t = crate::theme();
        self.ring = crate::layout::resolve_color(if props.invalid {
            t.destructive.with_alpha(0.2)
        } else {
            t.ring.with_alpha(0.5)
        });
        let text = props.textbox.text.clone();
        let placeholder = std::mem::take(&mut props.textbox.placeholder);
        let padding = props.textbox.padding;
        let mut placeholder_style = props.textbox.style.clone();
        placeholder_style.color = t.muted_foreground;
        let response = self.inner.update(props.textbox);
        let empty = response.text.as_deref().unwrap_or(&text).is_empty();
        // Yakui dims placeholders toward its fill in linear space. Keep its native
        // editor and cursor, but compose the semantic placeholder as another child.
        // The child remains at a stable call site even when the editor is nonempty.
        yakui::pad(padding, || {
            RenderText::with_style(
                if empty { placeholder } else { String::new() },
                placeholder_style,
            )
            .show();
        });
        response
    }
    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        if self.disabled {
            if ctx.input.focus() == Some(ctx.dom.current()) {
                ctx.input.set_focus(None, false);
            }
            self.default_layout(ctx, constraints)
        } else {
            self.inner.layout(ctx, constraints)
        }
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        if self.focused && !self.disabled && !self.grouped {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            let outer = yakui::Rect::from_pos_size(
                rect.pos() - Vec2::splat(3.),
                rect.size() + Vec2::splat(6.),
            );
            crate::icons::outline(&mut ctx, outer, self.radius + 3., 3., self.ring);
        }
        if self.disabled {
            self.default_paint(ctx)
        } else {
            self.inner.paint(ctx)
        }
    }
    fn event_interest(&self) -> EventInterest {
        if self.disabled {
            EventInterest::empty()
        } else {
            self.inner.event_interest()
        }
    }
    fn focus_policy(&self) -> FocusPolicy {
        if self.disabled {
            FocusPolicy::empty()
        } else {
            self.inner.focus_policy()
        }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::FocusChanged { focused, .. } = event {
            self.focused = *focused;
            return self.inner.event(ctx, event);
        }
        if self.disabled {
            return EventResponse::Bubble;
        }
        if crate::overlay::escape(&ctx, event, self.dismiss.as_deref()) {
            return EventResponse::Sink;
        }
        if let Some(group) = &self.menu {
            if group.editor_key(&ctx, event) {
                return EventResponse::Sink;
            }
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
        if self.read_only
            && matches!(
                event,
                WidgetEvent::TextInput(..)
                    | WidgetEvent::KeyChanged {
                        key: KeyCode::Backspace
                            | KeyCode::Delete
                            | KeyCode::Enter
                            | KeyCode::KeyV
                            | KeyCode::KeyX,
                        down: true,
                        ..
                    }
            )
        {
            return EventResponse::Sink;
        }
        self.inner.event(ctx, event)
    }
}
