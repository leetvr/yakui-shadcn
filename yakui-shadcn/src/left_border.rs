use yakui::util::auto_builders;
use yakui_core::Response;
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};

/// A horizontal divider line. Will take up the whole width of the parent.
///
/// Responds with [DividerResponse].
#[derive(Debug)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct LeftBorder {
    /// The color of the divider.
    pub color: Color,
    /// The thickness of the divider.
    pub thickness: f32,
    /// The indent of the divider from the left.
    pub indent: f32,
    /// The indent of the divider from the right.
    pub end_indent: f32,
}

auto_builders!(LeftBorder {
    color: Color,
    thickness: f32,
    indent: f32,
    end_indent: f32,
});

impl LeftBorder {
    pub fn new(color: Color, thickness: f32) -> Self {
        Self {
            color,
            thickness,
            indent: 0.0,
            end_indent: 0.0,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<LeftBorderResponse> {
        yakui::util::widget::<LeftBorderWidget>(self)
    }
}

#[derive(Debug)]
pub struct LeftBorderWidget {
    props: LeftBorder,
}

pub type LeftBorderResponse = ();

impl Widget for LeftBorderWidget {
    type Props<'a> = LeftBorder;
    type Response = LeftBorderResponse;

    fn new() -> Self {
        Self {
            props: LeftBorder::new(Color::WHITE, 0.0),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, _: LayoutContext<'_>, input: Constraints) -> Vec2 {
        Vec2::new(
            self.props.indent + self.props.thickness + self.props.end_indent,
            input.min.y,
        )
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let id = ctx.dom.current();
        let Some(parent) = ctx.dom.get(id).unwrap().parent else {
            return;
        };
        let line_height = ctx.layout.get(parent).unwrap().rect.size().y;

        let outer_rect = ctx.layout.get(id).unwrap().rect;

        let line_pos = outer_rect.pos() + Vec2::new(self.props.indent, outer_rect.size().y);
        let line_size = Vec2::new(self.props.thickness, line_height);

        let mut line_rect = PaintRect::new(Rect::from_pos_size(line_pos, line_size));
        line_rect.color = self.props.color;
        line_rect.add(ctx.paint);
    }
}
