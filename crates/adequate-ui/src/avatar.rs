//! Circular native textures, caller-owned loading state, status badges and groups.
use crate::{Color, Icon, Vec2, theme};
use yakui::{
    Constraints, Rect, TextureId,
    paint::{PaintMesh, Vertex},
    widget::{LayoutContext, PaintContext, Widget},
    widgets::Scope,
};
/// The three upstream avatar diameters.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    #[default]
    Default,
    Lg,
}
impl AvatarSize {
    pub fn diameter(self) -> f32 {
        match self {
            Self::Sm => 24.,
            Self::Default => 32.,
            Self::Lg => 40.,
        }
    }
}
/// An application texture and its pixel dimensions. Loading/decoding stays in the app.
#[derive(Debug, Clone, Copy)]
pub struct AvatarImage {
    pub texture: TextureId,
    pub pixel_size: Vec2,
}
impl AvatarImage {
    pub fn new(texture: TextureId, pixel_size: Vec2) -> Self {
        Self {
            texture,
            pixel_size,
        }
    }
}
/// A bottom-right status indicator. Its ring uses the surrounding surface.
#[derive(Debug, Clone, Copy)]
pub struct AvatarBadge {
    pub color: Option<Color>,
    pub icon: Option<Icon>,
}
impl AvatarBadge {
    pub fn new() -> Self {
        Self {
            color: None,
            icon: None,
        }
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}
impl Default for AvatarBadge {
    fn default() -> Self {
        Self::new()
    }
}
/// A user image with fallback text. Pass `None` while an image is loading or failed.
/// ```no_run
/// use yakui_adequate_ui as ui;
/// ui::Avatar::new("CN").size(ui::AvatarSize::Lg)
///     .badge(ui::AvatarBadge::new()).show();
/// ```
#[derive(Debug, Clone)]
pub struct Avatar {
    fallback: String,
    image: Option<AvatarImage>,
    size: AvatarSize,
    diameter: Option<f32>,
    badge: Option<AvatarBadge>,
    count_icon: Option<Icon>,
    group_count: bool,
}
impl Avatar {
    pub fn new(fallback: impl Into<String>) -> Self {
        Self {
            fallback: fallback.into(),
            image: None,
            size: AvatarSize::Default,
            diameter: None,
            badge: None,
            count_icon: None,
            group_count: false,
        }
    }
    pub fn image(mut self, image: Option<AvatarImage>) -> Self {
        self.image = image;
        self
    }
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self.diameter = None;
        self
    }
    /// Override the token diameter for an application-specific avatar.
    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = Some(diameter);
        self
    }
    pub fn badge(mut self, badge: AvatarBadge) -> Self {
        self.badge = Some(badge);
        self
    }
    #[track_caller]
    pub fn show(self) {
        let t = theme();
        let diameter = self.diameter.unwrap_or(self.size.diameter());
        let diameter = if diameter.is_finite() {
            diameter.max(0.)
        } else {
            0.
        };
        let image = self
            .image
            .filter(|image| image.pixel_size.is_finite() && image.pixel_size.min_element() > 0.);
        let badge_size = match self.size {
            AvatarSize::Sm => 8.,
            AvatarSize::Default => 10.,
            AvatarSize::Lg => 12.,
        };
        let count_icon_size = match self.size {
            AvatarSize::Sm => 12.,
            AvatarSize::Default => 16.,
            AvatarSize::Lg => 20.,
        };
        let badge_icon = self
            .badge
            .and_then(|badge| badge.icon)
            .filter(|_| self.size != AvatarSize::Sm);
        let props = AvatarProps {
            diameter,
            image,
            fill: crate::layout::resolve_color(t.muted),
            border: crate::layout::resolve_color(t.border),
            background: crate::layout::surface_background(),
            grouped: yakui::util::read_scope::<AvatarGroupScope>().is_some(),
            badge: self.badge.map(|badge| {
                (
                    crate::layout::resolve_color(badge.color.unwrap_or(t.primary)),
                    badge_size,
                )
            }),
            main_child: image.is_none(),
            badge_icon: badge_icon.is_some(),
        };
        yakui::util::widget_children::<AvatarWidget, _>(
            || {
                if image.is_none() {
                    if let Some(icon) = self.count_icon {
                        crate::icon_colored(icon, count_icon_size, t.muted_foreground);
                    } else {
                        let size = if self.size == AvatarSize::Sm && !self.group_count {
                            12.
                        } else {
                            14.
                        };
                        let mut style = t.text_style(size, t.muted_foreground, 400);
                        style.line_height_override = Some(if size == 12. { 16. } else { 20. });
                        yakui::widgets::RenderText::with_style(self.fallback, style).show();
                    }
                }
                if let Some(icon) = badge_icon {
                    crate::icon_colored(icon, 8., t.primary_foreground);
                }
            },
            props,
        );
    }
}
/// A count or icon at the end of an overlapping avatar group.
#[derive(Debug, Clone)]
pub struct AvatarGroupCount(Avatar);
impl AvatarGroupCount {
    pub fn new(count: impl Into<String>) -> Self {
        let mut avatar = Avatar::new(count);
        avatar.group_count = true;
        Self(avatar)
    }
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.0 = self.0.size(size);
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.0.count_icon = Some(icon);
        self
    }
    #[track_caller]
    pub fn show(self) {
        self.0.show();
    }
}
/// The original fallback shorthand, also suitable for an app-specific diameter.
#[track_caller]
pub fn avatar(initials: impl Into<String>, diameter: f32) {
    Avatar::new(initials).diameter(diameter).show();
}
#[derive(Debug, Clone, Copy)]
struct AvatarProps {
    diameter: f32,
    image: Option<AvatarImage>,
    fill: Color,
    border: Color,
    background: Color,
    grouped: bool,
    badge: Option<(Color, f32)>,
    main_child: bool,
    badge_icon: bool,
}
#[derive(Debug)]
struct AvatarWidget(Option<AvatarProps>);
impl Widget for AvatarWidget {
    type Props<'a> = AvatarProps;
    type Response = ();
    fn new() -> Self {
        Self(None)
    }
    fn update(&mut self, props: AvatarProps) {
        self.0 = Some(props);
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let props = self.0.unwrap();
        let size = constraints.constrain(Vec2::splat(props.diameter));
        let children = &ctx.dom.get_current().children;
        if props.main_child {
            if let Some(&child) = children.first() {
                let measured = ctx.calculate_layout(child, Constraints::loose(size));
                ctx.layout.set_pos(child, (size - measured) / 2.);
            }
        }
        if props.badge_icon {
            if let Some(&child) = children.get(usize::from(props.main_child)) {
                let icon = ctx.calculate_layout(child, Constraints::loose(Vec2::splat(8.)));
                let badge = props.badge.unwrap().1;
                ctx.layout
                    .set_pos(child, size - Vec2::splat(badge / 2.) - icon / 2.);
            }
        }
        size
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        let props = self.0.unwrap();
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let diameter = rect.size().min_element();
        let circle = Rect::from_pos_size(rect.pos(), Vec2::splat(diameter));
        if props.grouped {
            yakui::shapes::RoundedRectangle::new(
                Rect::from_pos_size(
                    circle.pos() - Vec2::splat(2.),
                    circle.size() + Vec2::splat(4.),
                ),
                999.,
            )
            .color(props.background)
            .add(ctx.paint);
        }
        if let Some(image) = props.image {
            // A fan crops the texture to the circle. UVs crop the longer source
            // dimension symmetrically, matching object-cover without an image
            // decoder, mask texture or renderer-specific shader in the library.
            let crop = Vec2::splat(image.pixel_size.min_element()) / image.pixel_size;
            let mut vertices = vec![Vertex::new(
                circle.center(),
                Vec2::splat(0.5),
                Color::WHITE.to_linear(),
            )];
            let mut indices = Vec::new();
            for i in 0..64 {
                let angle = std::f32::consts::TAU * i as f32 / 64.;
                let unit = Vec2::new(angle.cos(), angle.sin());
                vertices.push(Vertex::new(
                    circle.center() + unit * diameter / 2.,
                    Vec2::splat(0.5) + unit * crop / 2.,
                    Color::WHITE.to_linear(),
                ));
            }
            for i in 0..64u16 {
                indices.extend([0, i + 1, (i + 1) % 64 + 1]);
            }
            let mut mesh = PaintMesh::new(vertices, indices);
            mesh.texture = Some((image.texture, Rect::from_pos_size(Vec2::ZERO, Vec2::ONE)));
            ctx.paint.add_mesh(mesh);
        } else {
            yakui::shapes::RoundedRectangle::new(circle, 999.)
                .color(props.fill)
                .add(ctx.paint);
        }
        crate::icons::outline(&mut ctx, circle, diameter / 2., 1., props.border);
        if let Some((color, size)) = props.badge {
            let badge = Rect::from_pos_size(circle.max() - Vec2::splat(size), Vec2::splat(size));
            yakui::shapes::RoundedRectangle::new(
                Rect::from_pos_size(
                    badge.pos() - Vec2::splat(2.),
                    badge.size() + Vec2::splat(4.),
                ),
                999.,
            )
            .color(props.background)
            .add(ctx.paint);
            yakui::shapes::RoundedRectangle::new(badge, 999.)
                .color(color)
                .add(ctx.paint);
        }
        self.default_paint(ctx);
    }
}
#[derive(Debug)]
struct AvatarGroupScope;
#[derive(Debug)]
struct GroupWidget {
    rtl: bool,
}
impl Widget for GroupWidget {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self { rtl: false }
    }
    fn update(&mut self, _: ()) {
        self.rtl = crate::advanced_layout::rtl();
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let mut width = 0f32;
        let mut height = 0f32;
        let mut positions = Vec::new();
        for &child in &ctx.dom.get_current().children {
            let size = ctx.calculate_layout(child, Constraints::loose(constraints.max));
            positions.push((child, width, size));
            width += size.x - 8.;
            height = height.max(size.y);
        }
        if !positions.is_empty() {
            width += 8.;
        }
        let size = constraints.constrain(Vec2::new(width.max(0.), height));
        for (child, x, measured) in positions {
            ctx.layout.set_pos(
                child,
                Vec2::new(
                    if self.rtl { size.x - x - measured.x } else { x },
                    (size.y - measured.y) / 2.,
                ),
            );
        }
        size
    }
}
/// Overlap avatars by 8px, with a surrounding-surface ring on each member.
#[track_caller]
pub fn avatar_group(children: impl FnOnce()) {
    Scope::new(AvatarGroupScope).show(|| {
        yakui::util::widget_children::<GroupWidget, _>(children, ());
    });
}
