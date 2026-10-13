//! Application values, grouped choices and native menu/overlay interaction.
use crate::{Color, ControlResponse, Icon, SelectState, Vec2};
use yakui::{
    Constraints,
    widget::{LayoutContext, PaintContext, Widget},
    widgets::{ConstrainedBox, Pad, RenderText},
};
/// A choice, group label or separator. Values need not match slice positions.
#[derive(Debug, Clone)]
pub struct SelectItem {
    pub value: Option<usize>,
    pub text: String,
    pub disabled: bool,
    pub icon: Option<Icon>,
    kind: ItemKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemKind {
    Choice,
    Label,
    Separator,
}
impl SelectItem {
    pub fn new(value: usize, text: impl Into<String>) -> Self {
        Self {
            value: Some(value),
            text: text.into(),
            disabled: false,
            icon: None,
            kind: ItemKind::Choice,
        }
    }
    /// A selectable empty value, such as "Select a fruit".
    pub fn empty(text: impl Into<String>) -> Self {
        Self {
            value: None,
            ..Self::new(0, text)
        }
    }
    pub fn label(text: impl Into<String>) -> Self {
        Self {
            kind: ItemKind::Label,
            ..Self::empty(text)
        }
    }
    pub fn separator() -> Self {
        Self {
            kind: ItemKind::Separator,
            ..Self::empty("")
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}
/// Default (32px) and compact (28px) triggers.
#[derive(Debug, Default, Clone, Copy)]
pub enum SelectSize {
    #[default]
    Default,
    Sm,
}
/// A native Select trigger. Keep its value/visibility in SelectState and render
/// SelectContent later at the application root, outside scroll clipping.
///
/// ```no_run
/// use yakui_shadcn as ui;
/// let items = [ui::SelectItem::label("Quality"),
///     ui::SelectItem::new(10, "Low"), ui::SelectItem::new(20, "High")];
/// let mut state = ui::SelectState::default();
/// ui::Select::new(&items).placeholder("Choose quality").width(192.).show(&mut state);
/// // At the root, after ordinary content:
/// if ui::SelectContent::new(&items).show(&mut state) {
///     let quality = state.selected; // Some(10), Some(20), or None
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Select<'a> {
    items: &'a [SelectItem],
    placeholder: String,
    value_text: Option<String>,
    width: Option<f32>,
    size: SelectSize,
    disabled: bool,
    invalid: bool,
}
impl<'a> Select<'a> {
    pub fn new(items: &'a [SelectItem]) -> Self {
        Self {
            items,
            placeholder: "Select...".into(),
            value_text: None,
            width: None,
            size: SelectSize::Default,
            disabled: false,
            invalid: false,
        }
    }
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// Override the trigger's displayed value, for example a currency symbol.
    pub fn value_text(mut self, text: impl Into<String>) -> Self {
        self.value_text = Some(text.into());
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
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
    pub fn show(self, state: &mut SelectState) -> ControlResponse {
        if self.disabled {
            state.popup.close();
        }
        let item = self
            .items
            .iter()
            .find(|item| item.kind == ItemKind::Choice && item.value == state.selected);
        let text = self
            .value_text
            .as_deref()
            .or(item.map(|item| item.text.as_str()))
            .unwrap_or(&self.placeholder);
        let t = crate::theme();
        let empty = state.selected.is_none();
        let height = if matches!(self.size, SelectSize::Sm) {
            28.
        } else {
            32.
        };
        let radius = if matches!(self.size, SelectSize::Sm) {
            t.radius * 0.8
        } else {
            t.radius
        };
        let mut response = ControlResponse::default();
        let mut build = || {
            response = crate::menu::select_trigger(state, || {
                crate::interaction::control(
                    crate::interaction::Control {
                        disabled: self.disabled,
                        radius,
                        ring: if self.invalid { Color::CLEAR } else { t.ring },
                        focus_size: None,
                    },
                    |control| {
                        let fill = if t.dark {
                            t.input
                                .with_alpha(if control.hovering { 0.075 } else { 0.045 })
                        } else {
                            Color::CLEAR
                        };
                        let border = if self.invalid {
                            t.destructive
                        } else if control.focus_visible {
                            t.ring
                        } else {
                            t.input
                        };
                        let foreground = if empty {
                            t.muted_foreground
                        } else {
                            t.foreground
                        };
                        let foreground = crate::layout::resolve_color(if self.disabled {
                            foreground.with_alpha(0.5)
                        } else {
                            foreground
                        });
                        ConstrainedBox::new(Constraints {
                            min: Vec2::new(0., height),
                            max: Vec2::new(f32::INFINITY, height),
                        })
                        .show(|| {
                            crate::button_group::input_surface(
                                fill,
                                if self.disabled {
                                    border.with_alpha(border.a as f32 / 255. * 0.5)
                                } else {
                                    border
                                },
                                radius,
                                || {
                                    yakui::util::widget_children::<SelectHalo, _>(
                                        || {
                                            yakui::pad(
                                                Pad {
                                                    left: 10.,
                                                    right: 8.,
                                                    top: (height - 20.) / 2.,
                                                    bottom: (height - 20.) / 2.,
                                                },
                                                || {
                                                    yakui::util::widget_children::<TriggerRow, _>(
                                                        || {
                                                            if let Some(icon) =
                                                                item.and_then(|item| item.icon)
                                                            {
                                                                crate::icon_colored(
                                                                    icon, 16., foreground,
                                                                );
                                                            }
                                                            single_line(|| {
                                                                RenderText::with_style(
                                                                    text,
                                                                    t.text_style(
                                                                        14., foreground, 400,
                                                                    ),
                                                                )
                                                                .show();
                                                            });
                                                            crate::icon_colored(
                                                                Icon::ChevronDown,
                                                                16.,
                                                                crate::layout::resolve_color(
                                                                    if self.disabled {
                                                                        t.muted_foreground
                                                                            .with_alpha(0.5)
                                                                    } else {
                                                                        t.muted_foreground
                                                                    },
                                                                ),
                                                            );
                                                        },
                                                        item.is_some_and(|item| {
                                                            item.icon.is_some()
                                                        }),
                                                    );
                                                },
                                            );
                                        },
                                        (
                                            radius,
                                            if self.invalid {
                                                crate::layout::resolve_color(
                                                    t.destructive.with_alpha(if t.dark {
                                                        0.4
                                                    } else {
                                                        0.2
                                                    }),
                                                )
                                            } else {
                                                Color::CLEAR
                                            },
                                        ),
                                    );
                                },
                            );
                        });
                    },
                )
            });
        };
        if let Some(width) = self.width {
            crate::width(width, build);
        } else {
            build();
        }
        response
    }
}
// Shape without wrapping, then clip to the width assigned by the trigger/row.
// Measuring the text against that width would wrap and alter select row height.
#[derive(Debug)]
struct SingleLine;
impl Widget for SingleLine {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        let child = ctx.dom.get_current().children[0];
        let size = ctx.calculate_layout(child, Constraints::loose(Vec2::new(f32::INFINITY, 20.)));
        ctx.layout.set_pos(child, Vec2::ZERO);
        input.constrain(size)
    }
}
#[track_caller]
pub(crate) fn single_line(children: impl FnOnce()) {
    yakui::util::widget_children::<SingleLine, _>(children, ());
}
#[derive(Debug)]
struct TriggerRow(bool);
impl Widget for TriggerRow {
    type Props<'a> = bool;
    type Response = ();
    fn new() -> Self {
        Self(false)
    }
    fn update(&mut self, icon: bool) {
        self.0 = icon;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let children = &ctx.dom.get_current().children;
        let last = *children.last().unwrap();
        let chevron = ctx.calculate_layout(last, Constraints::loose(Vec2::splat(16.)));
        let leading = if self.0 {
            ctx.calculate_layout(children[0], Constraints::loose(Vec2::splat(16.)))
        } else {
            Vec2::ZERO
        };
        let left = if self.0 { leading.x + 6. } else { 0. };
        let text = ctx.calculate_layout(
            children[usize::from(self.0)],
            Constraints::loose(Vec2::new(
                (input.max.x - left - chevron.x - 6.).max(0.),
                20.,
            )),
        );
        let size = input.constrain(Vec2::new(left + text.x + 6. + chevron.x, 20.));
        if self.0 {
            ctx.layout.set_pos(children[0], Vec2::new(0., 2.));
        }
        ctx.layout
            .set_pos(children[usize::from(self.0)], Vec2::new(left, 0.));
        ctx.layout.set_pos(last, Vec2::new(size.x - chevron.x, 2.));
        size
    }
}
#[derive(Debug)]
struct SelectHalo {
    radius: f32,
    color: Color,
}
impl Widget for SelectHalo {
    type Props<'a> = (f32, Color);
    type Response = ();
    fn new() -> Self {
        Self {
            radius: 10.,
            color: Color::CLEAR,
        }
    }
    fn update(&mut self, (radius, color): Self::Props<'_>) {
        self.radius = radius;
        self.color = color;
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        if self.color.a != 0 {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            crate::icons::outline(
                &mut ctx,
                yakui::Rect::from_pos_size(
                    rect.pos() - Vec2::splat(3.),
                    rect.size() + Vec2::splat(6.),
                ),
                self.radius + 3.,
                3.,
                self.color,
            );
        }
        self.default_paint(ctx);
    }
}
/// Select's grouped, scrollable popup. Values are committed only on activation.
#[derive(Debug, Clone, Copy)]
pub struct SelectContent<'a> {
    items: &'a [SelectItem],
    width: f32,
    align_item: bool,
    align: crate::PopoverAlign,
}
impl<'a> SelectContent<'a> {
    pub fn new(items: &'a [SelectItem]) -> Self {
        Self {
            items,
            width: 0.,
            align_item: true,
            align: crate::PopoverAlign::Start,
        }
    }
    /// Zero follows the trigger width, with the source's 144px minimum.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn align_item_with_trigger(mut self, align: bool) -> Self {
        self.align_item = align;
        self
    }
    pub fn align(mut self, align: crate::PopoverAlign) -> Self {
        self.align = align;
        self
    }
    #[track_caller]
    pub fn show(self, state: &mut SelectState) -> bool {
        let selected = self
            .items
            .iter()
            .position(|item| {
                item.kind == ItemKind::Choice && item.value == state.selected && !item.disabled
            })
            .or_else(|| {
                self.items
                    .iter()
                    .position(|item| item.kind == ItemKind::Choice && !item.disabled)
            });
        let mut items: Vec<_> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| match item.kind {
                ItemKind::Label => crate::MenuItem::label(&item.text),
                ItemKind::Separator => crate::MenuItem::separator(),
                ItemKind::Choice => {
                    let mut choice = crate::MenuItem::radio(
                        index as u64,
                        &item.text,
                        0,
                        item.value == state.selected,
                    )
                    .disabled(item.disabled);
                    choice.icon = item.icon;
                    choice
                }
            })
            .collect();
        let mut row_offset = 4.;
        if let Some(index) = selected {
            for item in &self.items[..index] {
                row_offset += match item.kind {
                    ItemKind::Label => 24.,
                    ItemKind::Separator => 9.,
                    ItemKind::Choice => 28.,
                };
            }
        }
        let aligned = self.align_item && selected.is_some();
        let last = state.request_last.take().unwrap_or(false);
        let mut action = None;
        crate::overlay::select_popup(
            &mut state.popup,
            crate::PopoverOptions {
                width: self.width,
                align: self.align,
                side_offset: if aligned { -30. - row_offset } else { 4. },
                ..Default::default()
            },
            aligned,
            |popup| {
                action = crate::menu::select_list(&mut items, selected, last);
                if action.is_some() {
                    popup.close();
                }
            },
        );
        if let Some(index) = action {
            let value = self.items[index as usize].value;
            let changed = state.selected != value;
            state.selected = value;
            changed
        } else {
            false
        }
    }
}
