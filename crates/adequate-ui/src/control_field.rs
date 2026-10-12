//! Native field geometry shared by checkbox and switch indicators.
use yakui::{
    Color, Constraints, Rect, Vec2,
    widget::{LayoutContext, PaintContext, Widget},
};
#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldControlLayoutProps {
    pub description: bool,
    pub rtl: bool,
    pub trailing: bool,
    pub indicator_top: f32,
}
#[derive(Debug, Default)]
pub(crate) struct FieldControlLayout {
    description: bool,
    rtl: bool,
    trailing: bool,
    indicator_top: f32,
}
impl Widget for FieldControlLayout {
    type Props<'a> = FieldControlLayoutProps;
    type Response = ();
    fn new() -> Self {
        Self::default()
    }
    fn update(&mut self, props: Self::Props<'_>) {
        self.description = props.description;
        self.rtl = props.rtl;
        self.trailing = props.trailing;
        self.indicator_top = props.indicator_top;
    }
    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let children = &ctx.dom.get_current().children;
        let indicator = children[0];
        let indicator_size = ctx.calculate_layout(indicator, Constraints::loose(constraints.max));
        let mut content = Vec2::ZERO;
        if let Some(&label) = children.get(1) {
            content = ctx.calculate_layout(
                label,
                Constraints::loose(Vec2::new(
                    (constraints.max.x - indicator_size.x - 8.).max(0.),
                    constraints.max.y,
                )),
            );
        }
        let size = constraints.constrain(Vec2::new(
            indicator_size.x
                + if children.len() > 1 {
                    8. + content.x
                } else {
                    0.
                },
            indicator_size.y.max(content.y),
        ));
        // Helper text may be several lines tall. The indicator stays at the first
        // line, and its own paint node owns the ring so RTL and wrapping cannot
        // move keyboard focus decoration into the middle of the entire field.
        let y = if self.description {
            self.indicator_top
        } else {
            (size.y - indicator_size.y) / 2.
        };
        let indicator_right = self.rtl != self.trailing;
        ctx.layout.set_pos(
            indicator,
            Vec2::new(
                if indicator_right {
                    size.x - indicator_size.x
                } else {
                    0.
                },
                y,
            ),
        );
        if let Some(&label) = children.get(1) {
            let x = if self.rtl {
                size.x
                    - content.x
                    - if indicator_right {
                        indicator_size.x + 8.
                    } else {
                        0.
                    }
            } else if indicator_right {
                0.
            } else {
                indicator_size.x + 8.
            };
            let y = if self.description {
                0.
            } else {
                (size.y - content.y) / 2.
            };
            ctx.layout.set_pos(label, Vec2::new(x, y));
        }
        size
    }
}
#[derive(Debug)]
pub(crate) struct IndicatorRing(Color, f32);
impl Widget for IndicatorRing {
    type Props<'a> = (Color, f32);
    type Response = ();
    fn new() -> Self {
        Self(Color::CLEAR, 0.)
    }
    fn update(&mut self, (color, radius): Self::Props<'_>) {
        self.0 = color;
        self.1 = radius;
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        if self.0.a != 0 {
            let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            crate::icons::outline(
                &mut ctx,
                Rect::from_pos_size(rect.pos() - Vec2::splat(3.), rect.size() + Vec2::splat(6.)),
                self.1 + 3.,
                3.,
                self.0,
            );
        }
        for &child in &ctx.dom.get_current().children {
            ctx.paint(child);
        }
    }
}
