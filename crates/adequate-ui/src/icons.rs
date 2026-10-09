//! Small Lucide stroke icons in native geometry; 24-unit view box, 2-unit stroke.
pub use crate::lucide_icons::Icon;
use yakui::{
    Color, Constraints, Vec2,
    paint::{PaintMesh, Vertex},
    widget::PaintContext,
};
#[derive(Debug)]
struct IconBounds;
impl yakui::widget::Widget for IconBounds {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn layout(&self, ctx: yakui::widget::LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        // Yakui merges meshes with the same clip into a u16-indexed draw call.
        // Keep icon geometry within its own bounds: a dense catalogue can then
        // render without overflowing that shared call, and normal paint order holds.
        ctx.layout.enable_clipping(ctx.dom);
        self.default_layout(ctx, constraints)
    }
}
/// Paint a pinned Lucide icon with the scoped foreground token.
#[track_caller]
pub fn icon(icon: Icon, size: f32) {
    icon_colored(icon, size, crate::theme().foreground);
}
/// Paint actual Lucide vector paths using native Yakui geometry.
#[track_caller]
pub fn icon_colored(icon: Icon, size: f32, color: Color) {
    let size = if size.is_finite() { size.max(0.) } else { 0. };
    let color = crate::layout::resolve_color(color);
    crate::size(Vec2::splat(size), || {
        yakui::util::widget_children::<IconBounds, _>(
            || {
                yakui::widgets::Canvas::new(move |ctx| {
                    if color.a == 0 {
                        return;
                    }
                    let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
                    let scale = rect.size().min_element() / 24.;
                    if scale <= 0. {
                        return;
                    }
                    let stroke = 2. * scale;
                    let color = color.to_linear();
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();
                    for path in icon.paths() {
                        for pair in path.windows(2) {
                            let a = rect.pos() + Vec2::from(pair[0]) * scale;
                            let b = rect.pos() + Vec2::from(pair[1]) * scale;
                            let delta = b - a;
                            if delta.length_squared() <= 1e-12 {
                                continue;
                            }
                            let normal = Vec2::new(-delta.y, delta.x).normalize() * stroke / 2.;
                            let base = vertices.len() as u16;
                            vertices.extend(
                                [a - normal, a + normal, b + normal, b - normal]
                                    .map(|p| Vertex::new(p, Vec2::ZERO, color)),
                            );
                            indices.extend([base, base + 1, base + 2, base + 2, base + 3, base]);
                        }
                        // Lucide's strokes have round caps and joins. Disks at the sampled
                        // vertices join those native quads without inventing a runtime SVG
                        // interpreter. Semantic colour alpha is resolved against the scoped
                        // surface before drawing, so overlaps retain a uniform tint.
                        for &point in *path {
                            let center = rect.pos() + Vec2::from(point) * scale;
                            let base = vertices.len() as u16;
                            vertices.push(Vertex::new(center, Vec2::ZERO, color));
                            for i in 0..12 {
                                let angle = std::f32::consts::TAU * i as f32 / 12.;
                                vertices.push(Vertex::new(
                                    center + Vec2::new(angle.cos(), angle.sin()) * stroke / 2.,
                                    Vec2::ZERO,
                                    color,
                                ));
                            }
                            for i in 0..12 {
                                indices.extend([base, base + 1 + i, base + 1 + (i + 1) % 12]);
                            }
                        }
                    }
                    if !vertices.is_empty() {
                        ctx.paint.add_mesh(PaintMesh::new(vertices, indices));
                    }
                })
                .show();
            },
            (),
        );
    });
}
pub(crate) fn line(ctx: &mut PaintContext<'_>, a: Vec2, b: Vec2, stroke: f32, color: Color) {
    let delta = b - a;
    if delta.length_squared() == 0.0 {
        return;
    }
    let normal = Vec2::new(-delta.y, delta.x).normalize() * stroke / 2.;
    let vertices = [a - normal, a + normal, b + normal, b - normal]
        .map(|p| Vertex::new(p, Vec2::ZERO, color.to_linear()));
    ctx.paint
        .add_mesh(PaintMesh::new(vertices, [0, 1, 2, 2, 3, 0]));
}
pub(crate) fn circle(
    ctx: &mut PaintContext<'_>,
    center: Vec2,
    radius: f32,
    stroke: f32,
    color: Color,
    start: f32,
    end: f32,
) {
    for i in 0..48 {
        let a = start + (end - start) * i as f32 / 48.;
        let b = start + (end - start) * (i + 1) as f32 / 48.;
        line(
            ctx,
            center + Vec2::new(a.cos(), a.sin()) * radius,
            center + Vec2::new(b.cos(), b.sin()) * radius,
            stroke,
            color,
        );
    }
}

/// Hollow rounded outline. Yakui's bordered rectangle paints a solid silhouette
/// underneath its inset fill; a focus ring must leave the interior untouched.
pub(crate) fn outline(
    ctx: &mut yakui::widget::PaintContext<'_>,
    rect: yakui::Rect,
    radius: f32,
    width: f32,
    color: yakui::Color,
) {
    use yakui::paint::{PaintMesh, Vertex};
    const STEPS: usize = 12;
    let contour = |rect: yakui::Rect, radius: f32| {
        let r = radius.max(0.).min(rect.size().min_element().max(0.) / 2.);
        let centers = [
            rect.pos() + yakui::Vec2::splat(r),
            yakui::Vec2::new(rect.max().x - r, rect.pos().y + r),
            rect.max() - yakui::Vec2::splat(r),
            yakui::Vec2::new(rect.pos().x + r, rect.max().y - r),
        ];
        let mut result = Vec::new();
        for (corner, center) in centers.iter().enumerate() {
            for step in 0..=STEPS {
                let angle = std::f32::consts::PI
                    + (corner as f32 + step as f32 / STEPS as f32) * std::f32::consts::FRAC_PI_2;
                result.push(*center + yakui::Vec2::new(angle.cos(), angle.sin()) * r);
            }
        }
        result
    };
    let width = width.max(0.).min(rect.size().min_element().max(0.) / 2.);
    let outer = contour(rect, radius);
    let inner = contour(
        yakui::Rect::from_pos_size(
            rect.pos() + yakui::Vec2::splat(width),
            rect.size() - yakui::Vec2::splat(width * 2.),
        ),
        radius - width,
    );
    let vertices: Vec<_> = outer
        .iter()
        .zip(inner.iter())
        .flat_map(|(a, b)| {
            [
                Vertex::new(*a, yakui::Vec2::ZERO, color.to_linear()),
                Vertex::new(*b, yakui::Vec2::ZERO, color.to_linear()),
            ]
        })
        .collect();
    let count = outer.len() as u16;
    let indices: Vec<u16> = (0..count)
        .flat_map(|i| {
            let a = i * 2;
            let b = ((i + 1) % count) * 2;
            [a, b, b + 1, a, b + 1, a + 1]
        })
        .collect();
    ctx.paint.add_mesh(PaintMesh::new(vertices, indices));
}
