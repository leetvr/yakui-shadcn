//! An explicit native toast queue; expiration and actions stay local to the application.
use crate::{Button, ButtonSize, Variant, Vec2, column, row, surface, theme};
use std::time::{Duration, Instant};
#[derive(Debug, Default, Clone, Copy)]
pub enum ToastKind {
    #[default]
    Default,
    Success,
    Error,
    Info,
    Warning,
    Loading,
}
#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub kind: ToastKind,
    pub action: Option<String>,
    pub duration: Option<Duration>,
    created: Instant,
}
#[derive(Debug, Default)]
pub struct ToastManager {
    pub toasts: Vec<Toast>,
    next_id: u64,
}
impl ToastManager {
    pub fn add(
        &mut self,
        title: impl Into<String>,
        description: impl Into<String>,
        kind: ToastKind,
    ) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.toasts.push(Toast {
            id,
            title: title.into(),
            description: description.into(),
            kind,
            action: None,
            duration: Some(Duration::from_secs(5)),
            created: Instant::now(),
        });
        id
    }
    pub fn close(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }
    pub fn update(&mut self) {
        self.toasts.retain(|t| {
            t.duration
                .is_none_or(|duration| t.created.elapsed() < duration)
        });
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct ToastResponse {
    pub action: Option<u64>,
    pub dismissed: Option<u64>,
}
#[derive(Debug)]
struct ToastViewport;
impl yakui::widget::Widget for ToastViewport {
    type Props<'a> = ();
    type Response = ();
    fn new() -> Self {
        Self
    }
    fn update(&mut self, _: ()) {}
    fn layout(&self, mut ctx: yakui::widget::LayoutContext<'_>, input: yakui::Constraints) -> Vec2 {
        ctx.layout.new_layer(ctx.dom);
        let size = input.max;
        for &child in &ctx.dom.get_current().children {
            let body = ctx.calculate_layout(
                child,
                yakui::Constraints {
                    min: Vec2::new((size.x - 32.).clamp(0., 356.), 0.),
                    max: Vec2::new((size.x - 32.).clamp(0., 356.), (size.y - 32.).max(0.)),
                },
            );
            ctx.layout
                .set_pos(child, (size - body - Vec2::splat(16.)).max(Vec2::ZERO));
        }
        size
    }
}
/// Render at the application root, after content and dialogs. Actions return toast IDs.
#[track_caller]
pub fn toast_viewport(manager: &mut ToastManager) -> ToastResponse {
    manager.update();
    let t = theme();
    let mut response = ToastResponse::default();
    yakui::util::widget_children::<ToastViewport, _>(
        || {
            column(8., || {
                for toast in &manager.toasts {
                    surface(t.popover, Some(t.border), t.radius, 12., || {
                        row(12., || {
                            if matches!(toast.kind, ToastKind::Loading) {
                                crate::spinner(16., 0.);
                            } else if matches!(toast.kind, ToastKind::Success) {
                                crate::icon(crate::Icon::Check, 16.);
                            }
                            column(4., || {
                                crate::label(&toast.title);
                                crate::muted(&toast.description);
                            });
                            if let Some(action) = &toast.action {
                                if Button::new(action).size(ButtonSize::Sm).show().clicked {
                                    response.action = Some(toast.id);
                                }
                            }
                            if Button::new("")
                                .variant(Variant::Ghost)
                                .size(ButtonSize::IconXs)
                                .icon(crate::Icon::X)
                                .show()
                                .clicked
                            {
                                response.dismissed = Some(toast.id);
                            }
                        })
                    });
                }
            })
        },
        (),
    );
    if let Some(id) = response.dismissed {
        manager.close(id);
    }
    response
}
