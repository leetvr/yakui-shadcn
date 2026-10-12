//! Native chart geometry and hit testing. Values and selection stay application-owned.
use yakui::{
    Color, Constraints, Rect, Vec2, WidgetId,
    event::{EventInterest, EventResponse, WidgetEvent},
    input::{KeyCode, MouseButton},
    paint::{PaintMesh, Vertex},
    widget::{EventContext, FocusPolicy, LayoutContext, PaintContext, Widget},
};
/// A named series with an explicit semantic colour.
#[derive(Debug, Clone)]
pub struct ChartSeries {
    pub label: String,
    pub color: Color,
}
impl ChartSeries {
    pub fn new(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color,
        }
    }
}
/// One X/category sample; values follow the series order.
#[derive(Debug, Clone)]
pub struct ChartPoint {
    pub label: String,
    pub values: Vec<f64>,
}
impl ChartPoint {
    pub fn new(label: impl Into<String>, values: impl Into<Vec<f64>>) -> Self {
        Self {
            label: label.into(),
            values: values.into(),
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ChartKind {
    #[default]
    Bar,
    Line,
    Area,
    Pie,
    Donut,
    Radar,
    Radial,
    Scatter,
}
/// A plotted chart with mouse/keyboard inspection and activation.
#[derive(Debug, Clone)]
pub struct Chart {
    pub data: Vec<ChartPoint>,
    pub series: Vec<ChartSeries>,
    pub kind: ChartKind,
    pub size: Vec2,
    pub stacked: bool,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct ChartResponse {
    pub hovered: Option<usize>,
    pub clicked: Option<usize>,
    pub id: Option<WidgetId>,
}
impl Chart {
    pub fn new(data: Vec<ChartPoint>, series: Vec<ChartSeries>) -> Self {
        Self {
            data,
            series,
            kind: ChartKind::Bar,
            size: Vec2::new(480., 240.),
            stacked: false,
        }
    }
    pub fn kind(mut self, kind: ChartKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self
    }
    #[track_caller]
    pub fn show(self) -> ChartResponse {
        let labels = self
            .data
            .iter()
            .map(|p| p.label.clone())
            .collect::<Vec<_>>();
        let series = self.series.clone();
        let width = self.size.x;
        let mut output = ChartResponse::default();
        crate::column(12., || {
            let response = yakui::util::widget::<PlotWidget>(self);
            let id = response.id;
            output = ChartResponse {
                id: Some(id),
                ..response.into_inner()
            };
            if !labels.is_empty() {
                crate::row(0., || {
                    let stride = labels.len().div_ceil(6);
                    for label in labels.iter().step_by(stride) {
                        crate::width(width / labels.len().div_ceil(stride) as f32, || {
                            crate::muted(label)
                        });
                    }
                });
            }
            crate::row(16., || {
                for series in series {
                    crate::row(6., || {
                        crate::size(Vec2::splat(8.), || {
                            yakui::widgets::RoundRect::new(2.)
                                .color(series.color)
                                .show();
                        });
                        crate::muted(series.label);
                    });
                }
            });
        });
        output
    }
}
fn finite(value: f64) -> f64 {
    if value.is_finite() { value } else { 0. }
}
fn value(point: &ChartPoint, series: usize) -> f64 {
    finite(point.values.get(series).copied().unwrap_or(0.))
}
fn range(chart: &Chart) -> (f64, f64) {
    let mut min = 0f64;
    let mut max = 0f64;
    for p in &chart.data {
        if chart.stacked {
            let positive: f64 = p.values.iter().map(|v| finite(*v).max(0.)).sum();
            let negative: f64 = p.values.iter().map(|v| finite(*v).min(0.)).sum();
            max = max.max(positive);
            min = min.min(negative);
        } else {
            for v in &p.values {
                max = max.max(finite(*v));
                min = min.min(finite(*v));
            }
        }
    }
    if min == max {
        max = min + 1.;
    }
    (min, max)
}
#[derive(Debug)]
struct PlotWidget {
    props: Chart,
    hovered: Option<usize>,
    clicked: Option<usize>,
    focused: bool,
    pointer_down: bool,
    palette: [Color; 5],
    grid: Color,
    background: Color,
}
impl Widget for PlotWidget {
    type Props<'a> = Chart;
    type Response = ChartResponse;
    fn new() -> Self {
        Self {
            props: Chart::new(Vec::new(), Vec::new()),
            hovered: None,
            clicked: None,
            focused: false,
            pointer_down: false,
            palette: crate::Theme::light().chart,
            grid: Color::GRAY,
            background: Color::WHITE,
        }
    }
    fn update(&mut self, props: Chart) -> ChartResponse {
        self.props = props;
        let t = crate::theme();
        self.palette = t.chart;
        self.grid = crate::layout::resolve_color(t.border);
        self.background = crate::layout::surface_background();
        self.hovered = self.hovered.filter(|i| *i < self.props.data.len());
        ChartResponse {
            hovered: self.hovered,
            clicked: self.clicked.take(),
            id: None,
        }
    }
    fn layout(&self, _ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        constraints.constrain(self.props.size.max(Vec2::ZERO))
    }
    fn paint(&self, mut ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let n = self.props.data.len();
        if n == 0 || rect.size().min_element() <= 0. {
            return;
        }
        let (min, max) = range(&self.props);
        let y = |v: f64| rect.max().y - ((v - min) / (max - min)) as f32 * rect.size().y;
        let color = |series: usize| {
            self.props
                .series
                .get(series)
                .map_or(self.palette[series % 5], |s| s.color)
        };
        if matches!(self.props.kind, ChartKind::Pie | ChartKind::Donut) {
            let total: f64 = self.props.data.iter().map(|p| value(p, 0).max(0.)).sum();
            if total <= 0. {
                return;
            }
            let center = rect.center();
            let radius = rect.size().min_element() * 0.45;
            let inner = if self.props.kind == ChartKind::Donut {
                radius * 0.6
            } else {
                0.
            };
            let mut start = -std::f32::consts::FRAC_PI_2;
            for (i, p) in self.props.data.iter().enumerate() {
                let angle = (value(p, 0).max(0.) / total) as f32 * std::f32::consts::TAU;
                let segments = ((angle * 20.).ceil() as usize).max(1);
                let radius = radius + if self.hovered == Some(i) { 3. } else { 0. };
                for j in 0..segments {
                    let a = start + angle * j as f32 / segments as f32;
                    let b = start + angle * (j + 1) as f32 / segments as f32;
                    let u = Vec2::new(a.cos(), a.sin());
                    let v = Vec2::new(b.cos(), b.sin());
                    quad(
                        &mut ctx,
                        [
                            center + u * inner,
                            center + u * radius,
                            center + v * radius,
                            center + v * inner,
                        ],
                        self.palette[i % 5],
                    );
                }
                start += angle;
            }
            return;
        }
        if self.props.kind == ChartKind::Radar {
            let center = rect.center();
            let radius = rect.size().min_element() * 0.45;
            for i in 0..n {
                let a = i as f32 * std::f32::consts::TAU / n as f32 - std::f32::consts::FRAC_PI_2;
                crate::icons::line(
                    &mut ctx,
                    center,
                    center + Vec2::new(a.cos(), a.sin()) * radius,
                    1.,
                    self.grid,
                );
            }
            for s in 0..self.props.series.len().max(1) {
                let points: Vec<_> = self
                    .props
                    .data
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let a = i as f32 * std::f32::consts::TAU / n as f32
                            - std::f32::consts::FRAC_PI_2;
                        center
                            + Vec2::new(a.cos(), a.sin())
                                * radius
                                * ((value(p, s) - min) / (max - min)) as f32
                    })
                    .collect();
                for i in 0..n {
                    let next = (i + 1) % n;
                    triangle(
                        &mut ctx,
                        [center, points[i], points[next]],
                        crate::layout::composite(color(s).with_alpha(0.2), self.background),
                    );
                    crate::icons::line(&mut ctx, points[i], points[next], 2., color(s));
                }
            }
            return;
        }
        if self.props.kind == ChartKind::Radial {
            let center = rect.center();
            let radius = rect.size().min_element() * 0.4;
            for (i, p) in self.props.data.iter().enumerate() {
                let r = (radius - i as f32 * 14.).max(4.);
                crate::icons::circle(
                    &mut ctx,
                    center,
                    r,
                    10.,
                    self.grid,
                    0.,
                    std::f32::consts::TAU,
                );
                crate::icons::circle(
                    &mut ctx,
                    center,
                    r,
                    10.,
                    self.palette[i % 5],
                    -std::f32::consts::FRAC_PI_2,
                    -std::f32::consts::FRAC_PI_2
                        + ((value(p, 0) - min) / (max - min)) as f32 * std::f32::consts::TAU,
                );
            }
            return;
        }
        for tick in 0..=4 {
            let y = rect.pos().y + rect.size().y * tick as f32 / 4.;
            crate::icons::line(
                &mut ctx,
                Vec2::new(rect.pos().x, y),
                Vec2::new(rect.max().x, y),
                1.,
                self.grid,
            );
        }
        let series = self.props.series.len().max(1);
        let cell = rect.size().x / n as f32;
        for s in 0..series {
            let mut previous = None::<Vec2>;
            for (i, p) in self.props.data.iter().enumerate() {
                let x = rect.pos().x + cell * (i as f32 + 0.5);
                let point = Vec2::new(x, y(value(p, s)));
                match self.props.kind {
                    ChartKind::Bar => {
                        let base = if self.props.stacked {
                            (0..s)
                                .map(|k| value(p, k))
                                .filter(|v| v.signum() == value(p, s).signum())
                                .sum()
                        } else {
                            0.
                        };
                        let end = base + value(p, s);
                        let top = y(base).min(y(end));
                        let height = (y(base) - y(end)).abs();
                        let bw = if self.props.stacked {
                            cell * 0.75
                        } else {
                            cell * 0.75 / series as f32
                        };
                        let bx = if self.props.stacked {
                            x - bw / 2.
                        } else {
                            x - cell * 0.375 + s as f32 * bw
                        };
                        yakui::shapes::RoundedRectangle::new(
                            Rect::from_pos_size(
                                Vec2::new(bx, top),
                                Vec2::new((bw - 1.).max(0.), height),
                            ),
                            3.,
                        )
                        .color(color(s))
                        .add(ctx.paint);
                    }
                    ChartKind::Line | ChartKind::Area => {
                        if let Some(prev) = previous {
                            if self.props.kind == ChartKind::Area {
                                quad(
                                    &mut ctx,
                                    [
                                        Vec2::new(prev.x, y(0.)),
                                        prev,
                                        point,
                                        Vec2::new(point.x, y(0.)),
                                    ],
                                    crate::layout::composite(
                                        color(s).with_alpha(0.2),
                                        self.background,
                                    ),
                                );
                            }
                            crate::icons::line(&mut ctx, prev, point, 2., color(s));
                        }
                        yakui::shapes::Circle::new(
                            point,
                            if self.hovered == Some(i) { 4. } else { 2. },
                        )
                        .color(color(s))
                        .add(ctx.paint);
                    }
                    ChartKind::Scatter => {
                        yakui::shapes::Circle::new(point, 4.)
                            .color(color(s))
                            .add(ctx.paint);
                    }
                    _ => {}
                }
                previous = Some(point);
            }
        }
    }
    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL | EventInterest::FOCUSED_KEYBOARD
    }
    fn focus_policy(&self) -> FocusPolicy {
        FocusPolicy::SEQUENTIAL | FocusPolicy::POINTER
    }
    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        let n = self.props.data.len();
        if n == 0 {
            return EventResponse::Bubble;
        }
        match event {
            WidgetEvent::MouseMoved(Some(pos)) => {
                let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
                if !rect.contains_point(*pos) {
                    self.hovered = None;
                    return EventResponse::Bubble;
                }
                if matches!(self.props.kind, ChartKind::Pie | ChartKind::Donut) {
                    let v = *pos - rect.center();
                    let radius = rect.size().min_element() * 0.45;
                    if v.length() > radius
                        || (self.props.kind == ChartKind::Donut && v.length() < radius * 0.6)
                    {
                        self.hovered = None;
                        return EventResponse::Bubble;
                    }
                    let angle = (v.y.atan2(v.x) + std::f32::consts::FRAC_PI_2)
                        .rem_euclid(std::f32::consts::TAU);
                    let total: f64 = self.props.data.iter().map(|p| value(p, 0).max(0.)).sum();
                    let mut sum = 0.;
                    self.hovered = None;
                    for (i, p) in self.props.data.iter().enumerate() {
                        sum += value(p, 0).max(0.) / total;
                        if angle as f64 / std::f64::consts::TAU <= sum {
                            self.hovered = Some(i);
                            break;
                        }
                    }
                } else {
                    self.hovered =
                        Some((((pos.x - rect.pos().x) / rect.size().x) * n as f32) as usize)
                            .map(|i| i.min(n - 1));
                }
            }
            WidgetEvent::MouseLeave => self.hovered = None,
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                ..
            } => {
                self.pointer_down = true;
                ctx.input.set_focus(Some(ctx.dom.current()), false);
                return EventResponse::Sink;
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: false,
                inside,
                ..
            } if self.pointer_down => {
                self.pointer_down = false;
                if *inside {
                    self.clicked = self.hovered;
                }
                return EventResponse::Sink;
            }
            WidgetEvent::FocusChanged { focused, .. } => {
                self.focused = *focused;
                if !focused {
                    self.pointer_down = false;
                }
                if *focused {
                    self.hovered = Some(0);
                }
            }
            WidgetEvent::KeyChanged {
                key, down: true, ..
            } => {
                match key {
                    KeyCode::ArrowLeft => {
                        self.hovered = Some(self.hovered.unwrap_or(0).saturating_sub(1))
                    }
                    KeyCode::ArrowRight => {
                        self.hovered = Some((self.hovered.unwrap_or(0) + 1).min(n - 1))
                    }
                    KeyCode::Home => self.hovered = Some(0),
                    KeyCode::End => self.hovered = Some(n - 1),
                    KeyCode::Enter | KeyCode::Space => self.clicked = self.hovered,
                    _ => return EventResponse::Bubble,
                }
                return EventResponse::Sink;
            }
            _ => {}
        }
        EventResponse::Bubble
    }
}
fn triangle(ctx: &mut PaintContext<'_>, positions: [Vec2; 3], color: Color) {
    ctx.paint.add_mesh(PaintMesh::new(
        positions.map(|p| Vertex::new(p, Vec2::ZERO, color.to_linear())),
        [0, 1, 2],
    ));
}
fn quad(ctx: &mut PaintContext<'_>, positions: [Vec2; 4], color: Color) {
    ctx.paint.add_mesh(PaintMesh::new(
        positions.map(|p| Vertex::new(p, Vec2::ZERO, color.to_linear())),
        [0, 1, 2, 2, 3, 0],
    ));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chart_range_handles_empty_nonfinite_negative_and_stacked_values() {
        let mut c = Chart::new(
            vec![
                ChartPoint::new("A", vec![2., 3.]),
                ChartPoint::new("B", vec![-4., f64::NAN]),
            ],
            vec![],
        );
        assert_eq!(range(&c), (-4., 3.));
        c.stacked = true;
        assert_eq!(range(&c), (-4., 5.));
        c.data.clear();
        assert_eq!(range(&c), (0., 1.));
    }
}
