//! Small Lucide stroke icons in native geometry; 24-unit view box, 2-unit stroke.
use yakui::{
    Color, Vec2,
    paint::{PaintMesh, Vertex},
    widget::PaintContext,
};
/// Icons used by the first-party examples.
#[derive(Debug, Clone, Copy)]
pub enum Icon {
    Check,
    ChevronDown,
    ChevronRight,
    ChevronLeft,
    Plus,
    X,
    Search,
    ArrowRight,
    Loader,
    Menu,
    Sun,
    Moon,
    Mail,
    Settings,
}
/// Paint an icon at the requested logical size using the current foreground token.
#[track_caller]
pub fn icon(icon: Icon, size: f32) {
    icon_colored(icon, size, crate::theme().foreground);
}
/// Paint a Lucide icon with an explicit colour.
#[track_caller]
pub fn icon_colored(icon: Icon, size: f32, color: Color) {
    crate::size(Vec2::splat(size), || {
        yakui::widgets::Canvas::new(move |ctx| {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            let scale = size / 24.0;
            let paths: &[&[(f32, f32)]] = match icon {
                Icon::Check => &[&[(20., 6.), (9., 17.), (4., 12.)]],
                Icon::ChevronDown => &[&[(6., 9.), (12., 15.), (18., 9.)]],
                Icon::ChevronRight => &[&[(9., 6.), (15., 12.), (9., 18.)]],
                Icon::ChevronLeft => &[&[(15., 6.), (9., 12.), (15., 18.)]],
                Icon::Plus => &[&[(12., 5.), (12., 19.)], &[(5., 12.), (19., 12.)]],
                Icon::X => &[&[(18., 6.), (6., 18.)], &[(6., 6.), (18., 18.)]],
                Icon::ArrowRight => &[
                    &[(5., 12.), (19., 12.)],
                    &[(12., 5.), (19., 12.), (12., 19.)],
                ],
                Icon::Menu => &[
                    &[(4., 6.), (20., 6.)],
                    &[(4., 12.), (20., 12.)],
                    &[(4., 18.), (20., 18.)],
                ],
                Icon::Mail => &[
                    &[
                        (4., 4.),
                        (20., 4.),
                        (22., 6.),
                        (22., 18.),
                        (20., 20.),
                        (4., 20.),
                        (2., 18.),
                        (2., 6.),
                        (4., 4.),
                    ],
                    &[(22., 6.), (12., 13.), (2., 6.)],
                ],
                Icon::Moon => &[&[
                    (20.9, 13.),
                    (18., 17.),
                    (13., 20.),
                    (8., 19.),
                    (4., 15.),
                    (3., 10.),
                    (6., 5.),
                    (11., 3.),
                    (10., 8.),
                    (12., 12.),
                    (16., 14.),
                    (20.9, 13.),
                ]],
                _ => &[],
            };
            for path in paths {
                for pair in path.windows(2) {
                    line(
                        ctx,
                        rect.pos() + Vec2::from(pair[0]) * scale,
                        rect.pos() + Vec2::from(pair[1]) * scale,
                        2.0 * scale,
                        color,
                    );
                }
            }
            match icon {
                Icon::Search => {
                    circle(
                        ctx,
                        rect.pos() + Vec2::new(11., 11.) * scale,
                        8. * scale,
                        2. * scale,
                        color,
                        0.,
                        std::f32::consts::TAU,
                    );
                    line(
                        ctx,
                        rect.pos() + Vec2::new(16.65, 16.65) * scale,
                        rect.pos() + Vec2::new(21., 21.) * scale,
                        2. * scale,
                        color,
                    );
                }
                Icon::Loader => circle(
                    ctx,
                    rect.pos() + Vec2::splat(12.) * scale,
                    9. * scale,
                    2. * scale,
                    color,
                    0.,
                    5.1,
                ),
                Icon::Sun | Icon::Settings => {
                    circle(
                        ctx,
                        rect.center(),
                        4. * scale,
                        2. * scale,
                        color,
                        0.,
                        std::f32::consts::TAU,
                    );
                    for i in 0..8 {
                        let a = i as f32 * std::f32::consts::TAU / 8.;
                        let v = Vec2::new(a.cos(), a.sin());
                        line(
                            ctx,
                            rect.center() + v * 8. * scale,
                            rect.center() + v * 10. * scale,
                            2. * scale,
                            color,
                        );
                    }
                }
                _ => {}
            }
        })
        .show();
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
