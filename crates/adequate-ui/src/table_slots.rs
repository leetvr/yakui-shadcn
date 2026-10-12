//! Composable table cells use shared column widths and native horizontal clipping.
use std::{cell::Cell, rc::Rc};
use yakui::{
    Color, Constraints, Vec2, WidgetId, context,
    event::{EventInterest, EventResponse, WidgetEvent},
    widget::{EventContext, LayoutContext, PaintContext, Widget},
    widgets::Scope,
};

/// A native table whose rows can contain any public component.
/// Column widths include cell padding. Wider tables scroll within their parent.
///
/// ```no_run
/// # use yakui_adequate_ui as ui;
/// # let mut selected = false;
/// ui::Table::new(&[40., 240.]).caption("Available users").show(|| {
///     ui::table_header(|| ui::table_row(false, || {
///         ui::table_head("Select");
///         ui::table_head("Name");
///     }));
///     ui::table_body(|| ui::table_row(selected, || {
///         ui::table_cell(|| { ui::Checkbox::new().show(&mut selected); });
///         ui::table_cell(|| ui::text("Sarah Chen"));
///     }));
/// });
/// ```
#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<f32>,
    caption: Option<String>,
}
impl Table {
    pub fn new(columns: &[f32]) -> Self {
        Self {
            columns: columns
                .iter()
                .map(|w| if w.is_finite() { w.max(0.) } else { 0. })
                .collect(),
            caption: None,
        }
    }
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }
    #[track_caller]
    pub fn show(self, children: impl FnOnce()) {
        let width: f32 = self.columns.iter().sum();
        yakui::util::widget_children::<TableViewport, _>(
            || {
                crate::width(width, || {
                    crate::column(16., || {
                        Scope::new(Columns(self.columns)).show(|| crate::column(0., children));
                        if let Some(caption) = self.caption {
                            crate::fill_width(|| {
                                crate::min_size(Vec2::ZERO, || crate::muted(caption))
                            });
                        }
                    })
                });
            },
            width,
        );
    }
}
#[derive(Debug)]
struct Columns(Vec<f32>);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SectionKind {
    Header,
    Body,
    Footer,
}
#[derive(Debug)]
struct Section {
    kind: SectionKind,
    last: Cell<Option<WidgetId>>,
}
#[derive(Debug)]
struct RowColumns {
    columns: Rc<Columns>,
    cursor: Cell<usize>,
}
#[track_caller]
fn section(kind: SectionKind, children: impl FnOnce()) {
    let section = Section {
        kind,
        last: Cell::new(None),
    };
    // The last body's row ID is written during construction, before layout and
    // paint. Every row reads that same section handle to omit only its last border.
    Scope::new(section).show(|| crate::column(0., children));
}
/// Header rows use 40px minimum height and medium-weight heading cells.
#[track_caller]
pub fn table_header(children: impl FnOnce()) {
    section(SectionKind::Header, children);
}
/// The last body row omits its bottom border, matching the upstream TableBody.
#[track_caller]
pub fn table_body(children: impl FnOnce()) {
    section(SectionKind::Body, children);
}
/// Footer rows use a muted surface, top border and medium-weight cell text.
#[track_caller]
pub fn table_footer(children: impl FnOnce()) {
    section(SectionKind::Footer, children);
}
/// One content row. Selection belongs to the application; hovering adds a tint.
#[track_caller]
pub fn table_row(selected: bool, children: impl FnOnce()) {
    let columns =
        yakui::util::read_scope::<Columns>().expect("table_row belongs inside Table::show");
    let section =
        yakui::util::read_scope::<Section>().expect("table_row belongs inside a table section");
    let dom = context::dom();
    let response = dom.begin_widget::<TableRowWidget>((selected, Rc::clone(&section)));
    let id = response.id;
    section.last.set(Some(id));
    let hovering = response.into_inner();
    let t = crate::theme();
    let background = if selected {
        t.muted
    } else if hovering || section.kind == SectionKind::Footer {
        crate::layout::resolve_color(t.muted.with_alpha(0.5))
    } else {
        crate::layout::surface_background()
    };
    crate::layout::with_surface_background(background, || {
        Scope::new(RowColumns {
            columns,
            cursor: Cell::new(0),
        })
        .show(|| crate::row(0., children));
    });
    dom.end_widget::<TableRowWidget>(id);
}
/// Alignment of a table cell's content within its padded width.
#[derive(Debug, Default, Clone, Copy)]
pub enum TableCellAlign {
    #[default]
    Start,
    Center,
    End,
}
/// A cell can span adjacent columns and align arbitrary controls or text.
#[derive(Debug, Default, Clone, Copy)]
pub struct TableCell {
    span: usize,
    align: TableCellAlign,
}
impl TableCell {
    pub fn new() -> Self {
        Self {
            span: 1,
            align: TableCellAlign::Start,
        }
    }
    pub fn span(mut self, span: usize) -> Self {
        self.span = span.max(1);
        self
    }
    pub fn align(mut self, align: TableCellAlign) -> Self {
        self.align = align;
        self
    }
    #[track_caller]
    pub fn show(self, children: impl FnOnce()) {
        let row =
            yakui::util::read_scope::<RowColumns>().expect("TableCell belongs inside table_row");
        let start = row.cursor.get();
        let end = start
            .saturating_add(self.span.max(1))
            .min(row.columns.0.len());
        assert!(start < end, "TableCell exceeds the table's column count");
        row.cursor.set(end);
        let width: f32 = row.columns.0[start..end].iter().sum();
        yakui::util::widget_children::<CellWidget, _>(children, (width, self.align));
    }
}
#[track_caller]
pub fn table_cell(children: impl FnOnce()) {
    TableCell::new().show(children);
}
/// A medium-weight heading cell. Use TableCell for headings with rich content.
#[track_caller]
pub fn table_head(text: impl Into<String>) {
    table_cell(|| table_cell_text(text, true));
}
/// Normal cell text, optionally medium-weight; footer text is medium by default.
#[track_caller]
pub fn table_cell_text(text: impl Into<String>, medium: bool) {
    let t = crate::theme();
    let footer =
        yakui::util::read_scope::<Section>().is_some_and(|s| s.kind == SectionKind::Footer);
    yakui::widgets::RenderText::with_style(
        text,
        t.text_style(14., t.foreground, if medium || footer { 500 } else { 400 }),
    )
    .show();
}
#[derive(Debug)]
struct CellWidget {
    width: f32,
    align: TableCellAlign,
}
impl Widget for CellWidget {
    type Props<'a> = (f32, TableCellAlign);
    type Response = ();
    fn new() -> Self {
        Self {
            width: 0.,
            align: TableCellAlign::Start,
        }
    }
    fn update(&mut self, (width, align): Self::Props<'_>) {
        self.width = width;
        self.align = align;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let width = self.width.min(input.max.x);
        let available = (width - 16.).max(0.);
        let mut content = Vec2::ZERO;
        for &child in &ctx.dom.get_current().children {
            content = content.max(
                ctx.calculate_layout(child, Constraints::loose(Vec2::new(available, input.max.y))),
            );
        }
        let size = input.constrain(Vec2::new(width, content.y + 16.));
        for &child in &ctx.dom.get_current().children {
            let child_size = ctx.layout.get(child).unwrap().rect.size();
            let x = match self.align {
                TableCellAlign::Start => 8.,
                TableCellAlign::Center => (width - child_size.x) / 2.,
                TableCellAlign::End => width - child_size.x - 8.,
            };
            ctx.layout
                .set_pos(child, Vec2::new(x.max(0.), (size.y - child_size.y) / 2.));
        }
        size
    }
}
#[derive(Debug)]
struct TableRowWidget {
    selected: bool,
    hovering: bool,
    section: Option<Rc<Section>>,
    border: Color,
    border_width: f32,
    fill: Color,
    hover_fill: Color,
}
impl Widget for TableRowWidget {
    type Props<'a> = (bool, Rc<Section>);
    type Response = bool;
    fn new() -> Self {
        Self {
            selected: false,
            hovering: false,
            section: None,
            border: Color::CLEAR,
            border_width: 1.,
            fill: Color::CLEAR,
            hover_fill: Color::CLEAR,
        }
    }
    fn update(&mut self, (selected, section): Self::Props<'_>) -> bool {
        self.selected = selected;
        let t = crate::theme();
        self.border = crate::layout::resolve_color(t.border);
        self.border_width = t.border_width.max(0.);
        self.fill = if selected {
            t.muted
        } else if section.kind == SectionKind::Footer {
            crate::layout::resolve_color(t.muted.with_alpha(0.5))
        } else {
            Color::CLEAR
        };
        self.hover_fill = crate::layout::resolve_color(t.muted.with_alpha(0.5));
        self.section = Some(section);
        self.hovering
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let child = ctx.dom.get_current().children[0];
        let natural = ctx.calculate_layout(child, Constraints::loose(input.max));
        let section = self.section.as_ref().unwrap();
        let border =
            if section.kind != SectionKind::Body || section.last.get() != Some(ctx.dom.current()) {
                self.border_width
            } else {
                0.
            };
        let height = (natural.y + border).max(if section.kind == SectionKind::Header {
            40.
        } else {
            0.
        });
        let size = input.constrain(Vec2::new(natural.x, height));
        ctx.layout.set_pos(
            child,
            Vec2::new(
                0.,
                (size.y - border - natural.y).max(0.) / 2.
                    + if section.kind == SectionKind::Footer {
                        border
                    } else {
                        0.
                    },
            ),
        );
        size
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let fill = if self.selected {
            self.fill
        } else if self.hovering {
            self.hover_fill
        } else {
            self.fill
        };
        if fill.a != 0 {
            let mut fill_rect = yakui::paint::PaintRect::new(rect);
            fill_rect.color = fill;
            fill_rect.add(ctx.paint);
        }
        for &child in &ctx.dom.get_current().children {
            ctx.paint(child);
        }
        let section = self.section.as_ref().unwrap();
        if section.kind != SectionKind::Body || section.last.get() != Some(ctx.dom.current()) {
            let y = if section.kind == SectionKind::Footer {
                rect.pos().y
            } else {
                rect.max().y - self.border_width
            };
            let mut border_rect = yakui::paint::PaintRect::new(yakui::Rect::from_pos_size(
                Vec2::new(rect.pos().x, y),
                Vec2::new(rect.size().x, self.border_width),
            ));
            border_rect.color = self.border;
            border_rect.add(ctx.paint);
        }
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_OUTSIDE
    }
    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => self.hovering = true,
            WidgetEvent::MouseLeave => self.hovering = false,
            _ => {}
        }
        EventResponse::Bubble
    }
}
#[derive(Debug)]
struct TableViewport {
    width: f32,
    offset: Cell<f32>,
    overflow: Cell<f32>,
    last_focus: Cell<Option<WidgetId>>,
}
impl Widget for TableViewport {
    type Props<'a> = f32;
    type Response = ();
    fn new() -> Self {
        Self {
            width: 0.,
            offset: Cell::new(0.),
            overflow: Cell::new(0.),
            last_focus: Cell::new(None),
        }
    }
    fn update(&mut self, width: f32) {
        self.width = width;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        let child = ctx.dom.get_current().children[0];
        let canvas = ctx.calculate_layout(
            child,
            Constraints {
                min: Vec2::new(self.width, 0.),
                max: Vec2::new(self.width, input.max.y),
            },
        );
        let size = input.constrain(canvas);
        let overflow = (canvas.x - size.x).max(0.);
        self.overflow.set(overflow);
        let mut offset = self.offset.get();
        // Focus coordinates must be measured in the unscrolled canvas. Reset the
        // previous frame's child translation before comparing its bounds.
        ctx.layout.set_pos(child, Vec2::ZERO);
        let focus = ctx.input.focus();
        if self.last_focus.replace(focus) != focus {
            if let Some(rect) = focus.and_then(|id| ctx.rect_relative_to(id, ctx.dom.current())) {
                if rect.pos().x < offset {
                    offset = rect.pos().x;
                } else if rect.max().x > offset + size.x {
                    offset = rect.max().x - size.x;
                }
            }
        }
        offset = offset.clamp(0., overflow);
        self.offset.set(offset);
        ctx.layout.set_pos(child, Vec2::new(-offset, 0.));
        size
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }
    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        if let WidgetEvent::MouseScroll { delta, .. } = event {
            let delta = if delta.x != 0. { delta.x } else { delta.y };
            let next = (self.offset.get() + delta).clamp(0., self.overflow.get());
            if next != self.offset.get() {
                self.offset.set(next);
                return EventResponse::Sink;
            }
        }
        EventResponse::Bubble
    }
}
