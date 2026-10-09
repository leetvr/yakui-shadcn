//! One editor and ordered adornments sharing a surface. Values remain caller-owned.
use crate::{Color, InputResponse};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use yakui::{
    Constraints, Vec2, WidgetId,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::MouseButton,
    widget::{EventContext, LayoutContext, PaintContext, Widget},
    widgets::{Pad, Scope},
};
/// Placement relative to the group's editor; declaration order does not affect placement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputGroupAlign {
    #[default]
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    Editor,
    Addon(InputGroupAlign),
}
#[derive(Debug)]
pub(crate) struct InputGroupContext {
    slots: RefCell<Vec<(WidgetId, Slot)>>,
    pub editor: Cell<Option<WidgetId>>,
    pub disabled: Cell<bool>,
    pub invalid: Cell<bool>,
    focused: Cell<bool>,
}
/// A shared input surface. Declare one Input and any inline/block addons directly inside.
///
/// ```no_run
/// use yakui_adequate_ui as ui;
/// let mut query = String::new();
/// ui::width(320., || ui::input_group(|| {
///     ui::input_group_input(&mut query, "Search...");
///     ui::input_group_addon(ui::InputGroupAlign::InlineStart, || {
///         ui::icon(ui::Icon::Search, 16.);
///     });
///     ui::input_group_addon(ui::InputGroupAlign::InlineEnd, || {
///         if ui::input_group_button("Go").show().clicked { /* use query */ }
///     });
/// }));
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct InputGroup {
    pub disabled: bool,
    pub invalid: bool,
}
impl InputGroup {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }
    #[track_caller]
    pub fn show(self, children: impl FnOnce()) {
        let t = crate::theme();
        let fill = if self.disabled {
            t.input.with_alpha(if t.dark { 0.12 } else { 0.5 })
        } else if t.dark {
            t.input.with_alpha(0.045)
        } else {
            Color::CLEAR
        };
        crate::button_group::input_surface(
            fill,
            if self.invalid { t.destructive } else { t.input },
            t.radius,
            || {
                // Only the outer input surface participates in a surrounding button
                // group. Embedded actions keep their own corners and hit targets.
                crate::button_group::ungrouped(|| {
                    Scope::new(InputGroupContext {
                        slots: RefCell::default(),
                        editor: Cell::new(None),
                        disabled: Cell::new(self.disabled),
                        invalid: Cell::new(self.invalid),
                        focused: Cell::new(false),
                    })
                    .show(|| {
                        let group = yakui::util::read_scope::<InputGroupContext>().unwrap();
                        yakui::util::widget_children::<Group, _>(children, group);
                    });
                });
            },
        );
    }
}
/// A shared input surface with default options.
#[track_caller]
pub fn input_group(children: impl FnOnce()) {
    InputGroup::new().show(children);
}
#[derive(Debug)]
struct Group {
    group: Option<Rc<InputGroupContext>>,
    radius: f32,
    ring: Color,
    destructive: Color,
}
impl Widget for Group {
    type Props<'a> = Rc<InputGroupContext>;
    type Response = ();
    fn new() -> Self {
        Self {
            group: None,
            radius: 8.,
            ring: Color::CLEAR,
            destructive: Color::CLEAR,
        }
    }
    fn update(&mut self, group: Self::Props<'_>) {
        self.group = Some(group);
        let t = crate::theme();
        self.radius = t.radius;
        self.ring = crate::layout::resolve_color(t.ring.with_alpha(0.5));
        self.destructive = crate::layout::resolve_color(t.destructive.with_alpha(0.2));
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let group = self.group.as_ref().unwrap();
        group.focused.set(
            group
                .editor
                .get()
                .is_some_and(|id| ctx.input.focus() == Some(id)),
        );
        let slots = group.slots.borrow();
        let width = if input.max.x.is_finite() {
            input.max.x
        } else {
            input.min.x.max(240.)
        };
        let mut sizes = vec![Vec2::ZERO; slots.len()];
        let mut start_width = 0.;
        let mut end_width = 0.;
        let mut top = 0.;
        let mut bottom = 0.;
        let mut inline_height = 32f32;
        // Measure adornments first so the editor receives exactly the remaining
        // width. Each child is laid out once: Yakui also registers input here.
        for (index, &(id, kind)) in slots.iter().enumerate() {
            if let Slot::Addon(align) = kind {
                let size =
                    ctx.calculate_layout(id, Constraints::loose(Vec2::new(width, input.max.y)));
                sizes[index] = size;
                match align {
                    InputGroupAlign::InlineStart => {
                        start_width += size.x;
                        inline_height = inline_height.max(size.y)
                    }
                    InputGroupAlign::InlineEnd => {
                        end_width += size.x;
                        inline_height = inline_height.max(size.y)
                    }
                    InputGroupAlign::BlockStart => top += size.y,
                    InputGroupAlign::BlockEnd => bottom += size.y,
                }
            }
        }
        for (index, &(id, kind)) in slots.iter().enumerate() {
            if kind == Slot::Editor {
                let available = (width - start_width - end_width).max(0.);
                sizes[index] = ctx.calculate_layout(
                    id,
                    Constraints {
                        min: Vec2::new(available, 0.),
                        max: Vec2::new(available, (input.max.y - top - bottom).max(0.)),
                    },
                );
                inline_height = inline_height.max(sizes[index].y);
            }
        }
        let size = input.constrain(Vec2::new(width, top + inline_height + bottom));
        let mut start = 0.;
        let mut end = width - end_width;
        let mut top_y = 0.;
        let mut bottom_y = top + inline_height;
        for (index, &(id, kind)) in slots.iter().enumerate() {
            let measured = sizes[index];
            let pos = match kind {
                Slot::Editor => Vec2::new(start_width, top + (inline_height - measured.y) / 2.),
                Slot::Addon(InputGroupAlign::InlineStart) => {
                    let p = Vec2::new(start, top + (inline_height - measured.y) / 2.);
                    start += measured.x;
                    p
                }
                Slot::Addon(InputGroupAlign::InlineEnd) => {
                    let p = Vec2::new(end, top + (inline_height - measured.y) / 2.);
                    end += measured.x;
                    p
                }
                Slot::Addon(InputGroupAlign::BlockStart) => {
                    let p = Vec2::new(0., top_y);
                    top_y += measured.y;
                    p
                }
                Slot::Addon(InputGroupAlign::BlockEnd) => {
                    let p = Vec2::new(0., bottom_y);
                    bottom_y += measured.y;
                    p
                }
            };
            ctx.layout.set_pos(id, pos);
        }
        size
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        let group = self.group.as_ref().unwrap();
        let focused = group.focused.get();
        if !group.disabled.get() && (focused || group.invalid.get()) {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            let outer = yakui::Rect::from_pos_size(
                rect.pos() - Vec2::splat(3.),
                rect.size() + Vec2::splat(6.),
            );
            crate::icons::outline(
                &mut ctx,
                outer,
                self.radius + 3.,
                3.,
                if group.invalid.get() {
                    self.destructive
                } else {
                    self.ring
                },
            );
        }
        self.default_paint(ctx);
    }
}
#[derive(Debug)]
struct GroupSlot {
    kind: Slot,
    group: Option<Rc<InputGroupContext>>,
}
impl Widget for GroupSlot {
    type Props<'a> = Slot;
    type Response = ();
    fn new() -> Self {
        Self {
            kind: Slot::Editor,
            group: None,
        }
    }
    fn update(&mut self, kind: Slot) {
        self.kind = kind;
        self.group = yakui::util::read_scope();
        if let Some(group) = &self.group {
            group
                .slots
                .borrow_mut()
                .push((yakui::context::dom().current(), kind));
        }
    }
    fn event_interest(&self) -> EventInterest {
        if matches!(self.kind, Slot::Addon(_)) {
            EventInterest::MOUSE_INSIDE
        } else {
            EventInterest::empty()
        }
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::MouseButtonChanged {
            button: MouseButton::One,
            down: true,
            inside: true,
            ..
        } = event
        {
            if let Some(group) = &self.group {
                if !group.disabled.get() {
                    if let Some(editor) = group.editor.get() {
                        ctx.input.set_focus(Some(editor), false);
                        return EventResponse::Sink;
                    }
                }
            }
        }
        EventResponse::Bubble
    }
}
/// Leading/trailing icons, text or actions. Clicking non-action content focuses the editor.
#[track_caller]
pub fn input_group_addon(align: InputGroupAlign, children: impl FnOnce()) {
    yakui::util::widget_children::<GroupSlot, _>(
        || {
            let padding = match align {
                InputGroupAlign::InlineStart => Pad {
                    left: 8.,
                    right: 0.,
                    top: 4.,
                    bottom: 4.,
                },
                InputGroupAlign::InlineEnd => Pad {
                    left: 0.,
                    right: 8.,
                    top: 4.,
                    bottom: 4.,
                },
                InputGroupAlign::BlockStart | InputGroupAlign::BlockEnd => Pad::balanced(10., 8.),
            };
            yakui::pad(padding, || crate::row(8., children));
        },
        Slot::Addon(align),
    );
}
/// Secondary prefix/suffix text.
#[track_caller]
pub fn input_group_text(text: impl Into<String>) {
    let t = crate::theme();
    yakui::widgets::RenderText::with_style(
        text,
        t.text_style(t.font_size, t.muted_foreground, 500),
    )
    .show();
}
/// Compact action; change variant/size/icon using the ordinary Button API.
pub fn input_group_button(text: impl Into<String>) -> crate::Button {
    crate::Button::new(text)
        .for_input_group()
        .variant(crate::Variant::Ghost)
        .size(crate::ButtonSize::Xs)
}
/// Single-line input, sharing its parent's border and focus ring.
#[track_caller]
pub fn input_group_input(value: &mut String, placeholder: &str) -> InputResponse {
    crate::Input::new(value).placeholder(placeholder).show()
}
/// Multiline editor with the same native selection/clipboard behaviour as Input.
#[track_caller]
pub fn input_group_textarea(value: &mut String, placeholder: &str) -> InputResponse {
    crate::Input::new(value)
        .placeholder(placeholder)
        .multiline(true)
        .show()
}
#[track_caller]
pub(crate) fn editor_surface(fill: Color, border: Color, radius: f32, children: impl FnOnce()) {
    if yakui::util::read_scope::<InputGroupContext>().is_some() {
        yakui::util::widget_children::<GroupSlot, _>(children, Slot::Editor);
    } else {
        crate::button_group::input_surface(fill, border, radius, children);
    }
}
