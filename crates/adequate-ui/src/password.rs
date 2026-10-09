//! A password is edited as graphemes and painted only as bullets. Native TextBox
//! exposes its shaped plaintext for painting, so this control keeps selection and
//! pointer geometry in the masked coordinate space instead.
use crate::editor::EditorProps;
use std::{cell::Cell, rc::Rc};
use unicode_segmentation::UnicodeSegmentation;
use yakui::{
    Color, Constraints, Rect, Vec2,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::{KeyCode, MouseButton},
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
    widgets::{RenderText, TextBoxResponse},
};
#[derive(Debug)]
pub(crate) struct Password {
    props: EditorProps,
    pending: Option<String>,
    cursor: usize,
    anchor: usize,
    dragging: bool,
    focused: bool,
    submitted: bool,
    lost_focus: bool,
    advance: Rc<Cell<f32>>,
    scroll: Rc<Cell<f32>>,
    ring: Color,
    dismiss: Option<Rc<crate::overlay::DismissContext>>,
}
impl Password {
    fn count(&self) -> usize {
        self.props.textbox.text.graphemes(true).count()
    }
    fn byte(&self, index: usize) -> usize {
        self.props
            .textbox
            .text
            .grapheme_indices(true)
            .nth(index)
            .map_or(self.props.textbox.text.len(), |(byte, _)| byte)
    }
    fn replace(&mut self, text: &str) {
        let lo = self.cursor.min(self.anchor);
        let hi = self.cursor.max(self.anchor);
        let range = self.byte(lo)..self.byte(hi);
        let end = range.start + text.len();
        self.props.textbox.text.replace_range(range, text);
        self.cursor = self.props.textbox.text[..end].graphemes(true).count();
        self.anchor = self.cursor;
        self.pending = Some(self.props.textbox.text.clone());
    }
    fn position(&self, ctx: &EventContext<'_>, position: Vec2) -> usize {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        (((position.x - rect.pos().x - 10. + self.scroll.get()) / self.advance.get())
            .round()
            .max(0.) as usize)
            .min(self.count())
    }
}
impl Widget for Password {
    type Props<'a> = EditorProps;
    type Response = TextBoxResponse;
    fn new() -> Self {
        Self {
            props: EditorProps {
                textbox: yakui::widgets::TextBox::new(""),
                disabled: false,
                read_only: false,
                invalid: false,
            },
            pending: None,
            cursor: 0,
            anchor: 0,
            dragging: false,
            focused: false,
            submitted: false,
            lost_focus: false,
            advance: Rc::new(Cell::new(7.)),
            scroll: Rc::new(Cell::new(0.)),
            ring: Color::GRAY,
            dismiss: None,
        }
    }
    fn update(&mut self, mut props: EditorProps) -> TextBoxResponse {
        self.dismiss = yakui::util::read_scope();
        let pending = self.pending.take();
        if let Some(text) = &pending {
            props.textbox.text = text.clone();
        }
        self.props = props;
        self.cursor = self.cursor.min(self.count());
        self.anchor = self.anchor.min(self.count());
        let t = crate::theme();
        self.ring = crate::layout::resolve_color(if self.props.invalid {
            t.destructive.with_alpha(0.2)
        } else {
            t.ring.with_alpha(0.5)
        });
        let mut style = self.props.textbox.style.clone();
        if self.props.textbox.text.is_empty() {
            style.color = t.muted_foreground;
        }
        yakui::util::widget_children::<MaskedViewport, _>(
            || {
                RenderText::with_style(
                    if self.props.textbox.text.is_empty() {
                        self.props.textbox.placeholder.clone()
                    } else {
                        "•".repeat(self.count())
                    },
                    style,
                )
                .show();
            },
            MaskGeometry {
                count: self.count(),
                cursor: self.cursor,
                advance: self.advance.clone(),
                scroll: self.scroll.clone(),
            },
        );
        TextBoxResponse {
            text: pending,
            activated: std::mem::take(&mut self.submitted),
            lost_focus: std::mem::take(&mut self.lost_focus),
        }
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        if self.focused && !self.props.disabled {
            ctx.input.enable_text_input();
        }
        if self.props.disabled && ctx.input.focus() == Some(ctx.dom.current()) {
            ctx.input.set_focus(None, false);
        }
        let child = ctx.dom.get_current().children[0];
        let text_size = ctx.calculate_layout(
            child,
            Constraints {
                min: Vec2::new((constraints.min.x - 20.).max(0.), 20.),
                max: Vec2::new((constraints.max.x - 20.).max(0.), 20.),
            },
        );
        let size = constraints.constrain(Vec2::new(text_size.x + 20., 32.));
        ctx.layout.set_pos(child, Vec2::new(10., 6.));
        size
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        if self.focused && !self.props.disabled {
            crate::icons::outline(
                &mut ctx,
                Rect::from_pos_size(rect.pos() - Vec2::splat(3.), rect.size() + Vec2::splat(6.)),
                self.props.textbox.radius + 3.,
                3.,
                self.ring,
            );
        }
        // Clip both masked glyphs and selection/caret to the input's inner area.
        let clip = Rect::from_pos_size(
            rect.pos() + Vec2::new(10., 0.),
            Vec2::new((rect.size().x - 20.).max(0.), rect.size().y),
        );

        if self.focused && self.cursor != self.anchor {
            let x = self.cursor.min(self.anchor) as f32 * self.advance.get() - self.scroll.get();
            let width = self.cursor.abs_diff(self.anchor) as f32 * self.advance.get();
            let mut selection = yakui::paint::PaintRect::new(Rect::from_pos_size(
                rect.pos() + Vec2::new(10. + x, 6.),
                Vec2::new(width, 20.),
            ));
            selection.color = self.props.textbox.selected_bg_color;
            selection.rect = selection.rect.constrain(clip);
            selection.add(ctx.paint);
        }
        for &child in &ctx.dom.get_current().children {
            ctx.paint(child);
        }
        if self.focused && !self.props.disabled {
            let mut caret = yakui::paint::PaintRect::new(Rect::from_pos_size(
                rect.pos()
                    + Vec2::new(
                        10. + self.cursor as f32 * self.advance.get() - self.scroll.get(),
                        6.,
                    ),
                Vec2::new(1., 20.),
            ));
            caret.color = self.props.textbox.cursor_color;
            caret.rect = caret.rect.constrain(clip);
            caret.add(ctx.paint);
        }
    }
    fn focus_policy(&self) -> FocusPolicy {
        if self.props.disabled {
            FocusPolicy::empty()
        } else {
            FocusPolicy::SEQUENTIAL | FocusPolicy::POINTER
        }
    }
    fn event_interest(&self) -> EventInterest {
        if self.props.disabled {
            EventInterest::empty()
        } else {
            EventInterest::MOUSE_ALL | EventInterest::FOCUSED_KEYBOARD
        }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::FocusChanged { focused, .. } = event {
            self.focused = *focused;
            if !focused {
                self.dragging = false;
                self.lost_focus = true;
            }
            return EventResponse::Sink;
        }
        if self.props.disabled {
            return EventResponse::Bubble;
        }
        if crate::overlay::escape(&ctx, event, self.dismiss.as_deref()) {
            self.dragging = false;
            return EventResponse::Sink;
        }
        match event {
            WidgetEvent::TextInput(c, modifiers)
                if !self.props.read_only
                    && !c.is_control()
                    && !modifiers.ctrl()
                    && !modifiers.meta() =>
            {
                self.replace(&c.to_string());
                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                position,
                modifiers,
                ..
            } => {
                self.cursor = self.position(&ctx, *position);
                if !modifiers.shift() {
                    self.anchor = self.cursor;
                }
                self.dragging = true;
                ctx.input.set_focus(Some(ctx.dom.current()), false);
                EventResponse::Sink
            }
            WidgetEvent::MouseMoved(Some(position)) if self.dragging => {
                self.cursor = self.position(&ctx, *position);
                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: false,
                ..
            } if self.dragging => {
                self.dragging = false;
                EventResponse::Sink
            }
            WidgetEvent::KeyChanged {
                key,
                down: true,
                modifiers,
            } => {
                let command = modifiers.ctrl() || modifiers.meta();
                match key {
                    KeyCode::KeyA if command => {
                        self.anchor = 0;
                        self.cursor = self.count();
                    }
                    KeyCode::KeyC | KeyCode::KeyX if command => {} // Password text never enters the clipboard.
                    KeyCode::KeyV if command && !self.props.read_only => {
                        let clipboard = ctx
                            .dom
                            .get_global_or_init(yakui_widgets::clipboard::ClipboardHolder::default);
                        if let Some(text) = clipboard.paste() {
                            self.replace(
                                &text.chars().filter(|c| !c.is_control()).collect::<String>(),
                            );
                        }
                    }
                    KeyCode::ArrowLeft => {
                        self.cursor = if !modifiers.shift() && self.anchor != self.cursor {
                            self.anchor.min(self.cursor)
                        } else {
                            self.cursor.saturating_sub(1)
                        };
                        if !modifiers.shift() {
                            self.anchor = self.cursor;
                        }
                    }
                    KeyCode::ArrowRight => {
                        self.cursor = if !modifiers.shift() && self.anchor != self.cursor {
                            self.anchor.max(self.cursor)
                        } else {
                            (self.cursor + 1).min(self.count())
                        };
                        if !modifiers.shift() {
                            self.anchor = self.cursor;
                        }
                    }
                    KeyCode::Home => {
                        self.cursor = 0;
                        if !modifiers.shift() {
                            self.anchor = self.cursor;
                        }
                    }
                    KeyCode::End => {
                        self.cursor = self.count();
                        if !modifiers.shift() {
                            self.anchor = self.cursor;
                        }
                    }
                    KeyCode::Backspace if !self.props.read_only => {
                        if self.cursor == self.anchor {
                            self.anchor = self.cursor.saturating_sub(1);
                        }
                        self.replace("");
                    }
                    KeyCode::Delete if !self.props.read_only => {
                        if self.cursor == self.anchor {
                            self.anchor = (self.cursor + 1).min(self.count());
                        }
                        self.replace("");
                    }
                    KeyCode::Enter => {
                        self.submitted = true;
                    }
                    _ => return EventResponse::Bubble,
                }
                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
    }
}

#[derive(Debug, Clone)]
struct MaskGeometry {
    count: usize,
    cursor: usize,
    advance: Rc<Cell<f32>>,
    scroll: Rc<Cell<f32>>,
}
#[derive(Debug)]
struct MaskedViewport {
    geometry: MaskGeometry,
}
impl Widget for MaskedViewport {
    type Props<'a> = MaskGeometry;
    type Response = ();
    fn new() -> Self {
        Self {
            geometry: MaskGeometry {
                count: 0,
                cursor: 0,
                advance: Rc::new(Cell::new(7.)),
                scroll: Rc::new(Cell::new(0.)),
            },
        }
    }
    fn update(&mut self, geometry: MaskGeometry) {
        self.geometry = geometry;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        let child = ctx.dom.get_current().children[0];
        let natural =
            ctx.calculate_layout(child, Constraints::loose(Vec2::new(f32::INFINITY, 20.)));
        let size = constraints.constrain(natural);
        if self.geometry.count > 0 {
            self.geometry
                .advance
                .set((natural.x / self.geometry.count as f32).max(1.));
        }
        let caret = self.geometry.cursor as f32 * self.geometry.advance.get();
        let offset = if caret < self.geometry.scroll.get() {
            caret
        } else if caret > self.geometry.scroll.get() + size.x {
            caret - size.x
        } else {
            self.geometry.scroll.get()
        };
        self.geometry
            .scroll
            .set(offset.max(0.).min((natural.x - size.x).max(0.)));
        ctx.layout
            .set_pos(child, Vec2::new(-self.geometry.scroll.get(), 0.));
        size
    }
}
