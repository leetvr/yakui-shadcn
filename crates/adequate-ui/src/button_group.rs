//! Joined controls through a small styling scope and ordinary Yakui layout.
use crate::{Color, Orientation, Vec2};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use yakui::{
    Border, BorderRadius, Constraints, WidgetId, context,
    widget::{LayoutContext, PaintContext, Widget},
    widgets::Scope,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemberKind {
    Control,
    Nested,
    Separator,
}
#[derive(Debug)]
struct GroupContext {
    orientation: Orientation,
    rtl: bool,
    radius: f32,
    members: RefCell<Vec<(WidgetId, MemberKind)>>,
    cross: Cell<f32>,
    border_width: f32,
}
impl GroupContext {
    fn register(&self, id: WidgetId, kind: MemberKind) {
        self.members.borrow_mut().push((id, kind));
    }
    fn radii(&self, id: WidgetId, radius: f32) -> BorderRadius {
        let members = self.members.borrow();
        let Some(index) = members.iter().position(|(member, _)| *member == id) else {
            return radius.into();
        };
        let first = index == 0;
        let last = index + 1 == members.len();
        match self.orientation {
            Orientation::Vertical => BorderRadius::new(
                if first { radius } else { 0. },
                if first { radius } else { 0. },
                if last { self.radius } else { 0. },
                if last { self.radius } else { 0. },
            ),
            Orientation::Horizontal => {
                let (left, right) = if self.rtl {
                    (last, first)
                } else {
                    (first, last)
                };
                BorderRadius::new(
                    if left { radius } else { 0. },
                    if right { self.radius } else { 0. },
                    if left { radius } else { 0. },
                    if right { self.radius } else { 0. },
                )
            }
        }
    }
}
/// Horizontal or vertical joined actions. State and callbacks remain in the app.
#[derive(Debug, Default, Clone, Copy)]
pub struct ButtonGroup {
    pub orientation: Orientation,
}
impl ButtonGroup {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    #[track_caller]
    pub fn show(self, children: impl FnOnce()) {
        let parent = yakui::util::read_scope::<GroupContext>();
        let context = GroupContext {
            orientation: self.orientation,
            rtl: crate::advanced_layout::rtl(),
            radius: crate::theme().radius,
            members: RefCell::default(),
            cross: Cell::new(0.),
            border_width: crate::theme().border_width,
        };
        Scope::new(context).show(|| {
            let group = yakui::util::read_scope::<GroupContext>().unwrap();
            let dom = context::dom();
            let response = dom.begin_widget::<GroupWidget>(group);
            if let Some(parent) = parent {
                parent.register(response.id, MemberKind::Nested);
            }
            children();
            dom.end_widget::<GroupWidget>(response.id);
        });
    }
}
/// The usual horizontal joined action group.
#[track_caller]
pub fn button_group(children: impl FnOnce()) {
    ButtonGroup::new().show(children);
}
#[derive(Debug)]
struct GroupWidget(Option<Rc<GroupContext>>);
impl Widget for GroupWidget {
    type Props<'a> = Rc<GroupContext>;
    type Response = ();
    fn new() -> Self {
        Self(None)
    }
    fn update(&mut self, props: Self::Props<'_>) {
        self.0 = Some(props);
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let group = self.0.as_ref().unwrap();
        let horizontal = group.orientation == Orientation::Horizontal;
        let members = group.members.borrow();
        let nested = members.iter().any(|(_, kind)| *kind == MemberKind::Nested);
        let mut position = 0f32;
        let mut cross = 0f32;
        let mut layouts = Vec::new();
        for (index, &child) in ctx.dom.get_current().children.iter().enumerate() {
            if index > 0 {
                position += if nested {
                    8.
                } else if members
                    .get(index - 1)
                    .is_some_and(|(_, kind)| *kind == MemberKind::Separator)
                    || members
                        .get(index)
                        .is_some_and(|(_, kind)| *kind == MemberKind::Separator)
                {
                    0.
                } else {
                    -group.border_width
                };
            }
            let size = ctx.calculate_layout(child, Constraints::loose(constraints.max));
            layouts.push((child, position, size));
            position += if horizontal { size.x } else { size.y };
            cross = cross.max(if horizontal { size.y } else { size.x });
        }
        group.cross.set(cross);
        let size = constraints.constrain(if horizontal {
            Vec2::new(position.max(0.), cross)
        } else {
            Vec2::new(cross, position.max(0.))
        });
        for (child, position, measured) in layouts {
            ctx.layout.set_pos(
                child,
                if horizontal {
                    Vec2::new(
                        if group.rtl {
                            size.x - position - measured.x
                        } else {
                            position
                        },
                        (size.y - measured.y) / 2.,
                    )
                } else {
                    Vec2::new((size.x - measured.x) / 2., position)
                },
            );
        }
        size
    }
}
#[derive(Debug, Clone, Copy)]
struct FrameProps {
    fill: Color,
    border: Option<Border>,
    radius: f32,
}
#[derive(Debug)]
struct GroupFrame {
    props: FrameProps,
    group: Option<Rc<GroupContext>>,
}
impl Widget for GroupFrame {
    type Props<'a> = FrameProps;
    type Response = ();
    fn new() -> Self {
        Self {
            props: FrameProps {
                fill: Color::CLEAR,
                border: None,
                radius: 0.,
            },
            group: None,
        }
    }
    fn update(&mut self, props: Self::Props<'_>) {
        self.props = props;
        self.group = yakui::util::read_scope();
        if let Some(group) = &self.group {
            group.register(context::dom().current(), MemberKind::Control);
        }
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let radius = self.group.as_ref().map_or_else(
            || self.props.radius.into(),
            |group| group.radii(ctx.dom.current(), self.props.radius),
        );
        yakui::shapes::RoundedRectangle::new(rect, radius)
            .color(self.props.fill)
            .border(self.props.border)
            .add(ctx.paint);
        self.default_paint(ctx);
    }
}
#[track_caller]
pub(crate) fn button_surface(
    radius: f32,
    fill: Color,
    border: Option<Border>,
    children: impl FnOnce(),
) {
    if yakui::util::read_scope::<GroupContext>().is_none() {
        yakui::widgets::RoundRect::new(radius)
            .color(fill)
            .border(border)
            .show_children(children);
    } else {
        yakui::util::widget_children::<GroupFrame, _>(
            children,
            FrameProps {
                fill,
                border,
                radius,
            },
        );
    }
}
#[track_caller]
pub(crate) fn input_surface(fill: Color, border: Color, radius: f32, children: impl FnOnce()) {
    if yakui::util::read_scope::<GroupContext>().is_none() {
        crate::surface(fill, Some(border), radius, 0., children);
        return;
    }
    let background = crate::layout::surface_background();
    let fill = if fill.a == 0 {
        background
    } else {
        crate::layout::composite(fill, background)
    };
    let border = Border::new(
        crate::layout::composite(border, fill),
        crate::theme().border_width,
    );
    button_surface(radius, fill, Some(border), || {
        crate::layout::with_surface_background(fill, children)
    });
}
/// Noninteractive prefix/suffix text, joined to surrounding controls.
#[track_caller]
pub fn button_group_text(text: impl Into<String>) {
    let t = crate::theme();
    input_surface(t.muted, t.border, t.radius, || {
        crate::min_size(Vec2::new(0., 32.), || {
            yakui::pad(yakui::widgets::Pad::horizontal(10.), || {
                let mut style = t.text_style(14., t.foreground, 500);
                style.line_height_override = Some(20.);
                yakui::widgets::RenderText::with_style(text, style).show();
            });
        })
    });
}
#[derive(Debug)]
struct GroupSeparator {
    group: Option<Rc<GroupContext>>,
    color: Color,
}
impl Widget for GroupSeparator {
    type Props<'a> = Color;
    type Response = ();
    fn new() -> Self {
        Self {
            group: None,
            color: Color::CLEAR,
        }
    }
    fn update(&mut self, color: Color) {
        self.color = color;
        self.group = yakui::util::read_scope();
        if let Some(group) = &self.group {
            group.register(context::dom().current(), MemberKind::Separator);
        }
    }
    fn layout(&self, _: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        constraints.constrain(
            if self
                .group
                .as_ref()
                .is_some_and(|g| g.orientation == Orientation::Vertical)
            {
                Vec2::new(0., 1.)
            } else {
                Vec2::new(1., 0.)
            },
        )
    }
    fn paint(&self, ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let vertical = self
            .group
            .as_ref()
            .is_none_or(|group| group.orientation == Orientation::Horizontal);
        let cross = self.group.as_ref().map_or(32., |group| group.cross.get());
        let (offset, size) = if vertical {
            (
                Vec2::new(0., -cross / 2. + 1.),
                Vec2::new(1., (cross - 2.).max(0.)),
            )
        } else {
            (
                Vec2::new(-cross / 2. + 1., 0.),
                Vec2::new((cross - 2.).max(0.), 1.),
            )
        };
        let mut line =
            yakui::paint::PaintRect::new(yakui::Rect::from_pos_size(rect.pos() + offset, size));
        line.color = self.color;
        line.add(ctx.paint);
    }
}
/// A separator between joined actions; follows the surrounding group's axis.
#[track_caller]
pub fn button_group_separator() {
    yakui::util::widget::<GroupSeparator>(crate::layout::resolve_color(crate::theme().input));
}
