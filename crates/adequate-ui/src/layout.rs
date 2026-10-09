//! Small composition helpers using Yakui's own layout engine.
use yakui::{
    Border, Color, Constraints, CrossAxisAlignment, MainAxisSize, Vec2,
    widgets::{ConstrainedBox, List, Pad, RoundRect},
};
/// Horizontal content with the given gap, tightly sized to its children.
#[track_caller]
pub fn row(gap: f32, children: impl FnOnce()) {
    let props = List::row()
        .main_axis_size(MainAxisSize::Min)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .item_spacing(gap);
    yakui::util::widget_children::<crate::advanced_layout::RowWidget, _>(children, props);
}
/// Vertical content with the given gap, tightly sized to its children.
#[track_caller]
pub fn column(gap: f32, children: impl FnOnce()) {
    List::column()
        .main_axis_size(MainAxisSize::Min)
        .item_spacing(gap)
        .show(children);
}
/// A bounded content region. The parent can shrink it when space is constrained.
#[track_caller]
pub fn width(width: f32, children: impl FnOnce()) {
    ConstrainedBox::new(Constraints {
        min: Vec2::new(width, 0.0),
        max: Vec2::new(width, f32::INFINITY),
    })
    .show(children);
}
/// Reserve an exact logical size within the parent's constraints.
#[track_caller]
pub fn size(size: Vec2, children: impl FnOnce()) {
    ConstrainedBox::new(Constraints::tight(size)).show(children);
}
/// A rounded painted surface containing normal Yakui children.
#[track_caller]
pub fn surface(
    fill: Color,
    border: Option<Color>,
    radius: f32,
    padding: f32,
    children: impl FnOnce(),
) {
    let t = crate::theme();
    let background = surface_background();
    let resolved = if fill.a == 0 {
        background
    } else {
        composite(fill, background)
    };
    let border = border.map(|c| Border::new(composite(c, resolved), t.border_width));
    RoundRect::new(radius)
        // Yakui paints a solid border silhouette before the inset fill. A clear
        // interior would expose that silhouette across the whole surface.
        .color(if fill.a == 0 && border.is_none() {
            Color::CLEAR
        } else {
            resolved
        })
        .border(border)
        .show_children(|| {
            yakui::widgets::Scope::new(SurfaceBackground(resolved)).show(|| {
                yakui::pad(Pad::all(padding), children);
            });
        });
}
/// Clip and vertically scroll the children in a bounded region.
#[track_caller]
pub fn scroll_area(height: f32, children: impl FnOnce()) {
    ConstrainedBox::new(Constraints {
        min: Vec2::ZERO,
        max: Vec2::new(f32::INFINITY, height),
    })
    .show(|| {
        yakui::widgets::Scrollable::vertical().show(children);
    });
}

#[derive(Debug, Clone, Copy)]
struct SurfaceBackground(Color);
/// The opaque surface behind a component. CSS token alpha is composited in sRGB,
/// whereas game renderers blend in linear space; resolve semantic tints before
/// emitting paint so the same theme looks consistent on WGPU and Vulkan.
pub(crate) fn surface_background() -> Color {
    yakui::util::read_scope::<SurfaceBackground>()
        .map_or_else(|| crate::theme().background, |s| s.0)
}
pub(crate) fn composite(front: Color, back: Color) -> Color {
    let a = front.a as f32 / 255.;
    Color::rgb(
        (front.r as f32 * a + back.r as f32 * (1. - a)).round() as u8,
        (front.g as f32 * a + back.g as f32 * (1. - a)).round() as u8,
        (front.b as f32 * a + back.b as f32 * (1. - a)).round() as u8,
    )
}
pub(crate) fn resolve_color(color: Color) -> Color {
    if color.a == 0 {
        color
    } else {
        composite(color, surface_background())
    }
}

#[derive(Debug)]
struct FillWidth;
impl yakui::widget::Widget for FillWidth {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn layout(&self, ctx: yakui::widget::LayoutContext<'_>, mut constraints: Constraints) -> Vec2 {
        if constraints.max.x.is_finite() {
            constraints.min.x = constraints.max.x;
        }
        self.default_layout(ctx, constraints)
    }
}
/// Fill the available horizontal space while keeping content-driven height.
#[track_caller]
pub fn fill_width(children: impl FnOnce()) {
    yakui::util::widget_children::<FillWidth, _>(children, ());
}

#[derive(Debug)]
struct MinimumSize {
    minimum: Vec2,
}
impl yakui::widget::Widget for MinimumSize {
    type Props<'a> = Vec2;
    type Response = ();
    fn new() -> Self {
        Self {
            minimum: Vec2::ZERO,
        }
    }
    fn update(&mut self, minimum: Vec2) {
        self.minimum = minimum;
    }
    fn layout(&self, mut ctx: yakui::widget::LayoutContext<'_>, input: Constraints) -> Vec2 {
        let children = ctx.dom.get_current().children.clone();
        let mut natural = Vec2::ZERO;
        for &child in &children {
            natural = natural.max(ctx.calculate_layout(child, Constraints::loose(input.max)));
        }
        let size = input.constrain(natural.max(self.minimum));
        for &child in &children {
            let child_size = ctx.layout.get(child).unwrap().rect.size();
            ctx.layout.set_pos(child, (size - child_size) / 2.);
        }
        size
    }
}
/// A centered region with a minimum size; larger content expands the region normally.
#[track_caller]
pub fn min_size(minimum: Vec2, children: impl FnOnce()) {
    yakui::util::widget_children::<MinimumSize, _>(children, minimum);
}

/// Retain state on the component's container node rather than inserting an empty
/// state sibling into the caller's List (where it would consume item spacing).
#[track_caller]
pub(crate) fn with_state<T: 'static, R>(
    initial: impl FnOnce() -> T + 'static,
    children: impl FnOnce(yakui::widgets::StateResponse<T>) -> R,
) -> R {
    let dom = yakui::context::dom();
    let response =
        dom.begin_widget::<yakui::widgets::StateWidget<T>>(yakui::widgets::State::new(initial));
    let id = response.id;
    let result = children(response.into_inner());
    dom.end_widget::<yakui::widgets::StateWidget<T>>(id);
    result
}
