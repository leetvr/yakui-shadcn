//! Small composition helpers using Yakui's own layout engine.
use yakui::{
    Border, Color, Constraints, CrossAxisAlignment, MainAxisSize, Vec2,
    widgets::{ConstrainedBox, List, Pad, RoundRect},
};
/// Horizontal content with the given gap, tightly sized to its children.
#[track_caller]
pub fn row(gap: f32, children: impl FnOnce()) {
    List::row()
        .main_axis_size(MainAxisSize::Min)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .item_spacing(gap)
        .show(children);
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
        .color(if fill.a == 0 { Color::CLEAR } else { resolved })
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
